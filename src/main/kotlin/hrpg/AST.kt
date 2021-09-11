package hrpg

import hrpg.antlr.HRPGBaseVisitor
import hrpg.antlr.HRPGParser
import org.antlr.v4.runtime.tree.TerminalNode

sealed interface BaseNode {
    val comment: String
}

sealed interface Node : BaseNode {
    val binding: TerminalNode?

    fun tryBinding(binding: TerminalNode?): BaseNode
}

sealed interface NodeContainer : Node

// rule_body
data class Alternatives(override val binding: TerminalNode?, val nodes: List<Node>) : NodeContainer {
    override val comment: String
        get() = nodes.joinToString(" | ") { it.comment }

    override fun tryBinding(binding: TerminalNode?): BaseNode = copy(binding = binding)
}

// rule_piece
data class MultipartBody(override val binding: TerminalNode?, val nodes: List<Node>) : NodeContainer {
    override val comment: String
        get() = nodes.joinToString(" ") {
            when (it) {
                is Alternatives -> "(${it.comment})"
                else -> it.comment
            }
        }

    override fun tryBinding(binding: TerminalNode?): BaseNode = copy(binding = binding)
}

// rule_part
data class ZeroOrMore(override val binding: TerminalNode?, val node: Node) : NodeContainer {
    override val comment: String
        // TODO: Generate parens for all containers, but to be really accurate, we'd
        // need to parse the parens and track whether we saw them or not
        get() = if (node is NodeContainer) "(${node.comment})*" else node.comment + "*"

    override fun tryBinding(binding: TerminalNode?): BaseNode = copy(binding = binding)
}

// rule_part
data class OneOrMore(override val binding: TerminalNode?, val node: Node) : NodeContainer {
    override val comment: String
        // TODO: Generate parens for all containers, but to be really accurate, we'd
        // need to parse the parens and track whether we saw them or not
        get() = if (node is NodeContainer) "(${node.comment})+" else node.comment + "+"

    override fun tryBinding(binding: TerminalNode?): BaseNode = copy(binding = binding)
}

// rule_part
data class ZeroOrOne(override val binding: TerminalNode?, val node: Node, val brackets: Boolean) : NodeContainer {
    override val comment: String
        // TODO: Generate parens for all containers, but to be really accurate, we'd
        // need to parse the parens and track whether we saw them or not
        get() = if (brackets) "[" + node.comment + "]" else {
            if (node is NodeContainer) "(${node.comment})?" else node.comment + "?"
        }

    override fun tryBinding(binding: TerminalNode?): BaseNode = copy(binding = binding)
}

// RULE_NAME
data class RuleRef(override val binding: TerminalNode?, val name: TerminalNode) : Node {
    override val comment: String
        get() = name.text

    override fun tryBinding(binding: TerminalNode?): BaseNode = copy(binding = binding)
}

// TOKEN_NAME
data class TokenRef(
    override val binding: TerminalNode?, val name: TerminalNode, val replacedLit: TerminalNode? = null
) : Node {
    override val comment: String
        get() = if (replacedLit != null) replacedLit.text else name.text

    override fun tryBinding(binding: TerminalNode?): BaseNode = copy(binding = binding)
}

// TOKEN_LIT
data class TokenLit(override val binding: TerminalNode?, val literal: TerminalNode) : Node {
    override val comment: String
        get() = "\"" + literal.text + "\""

    override fun tryBinding(binding: TerminalNode?): BaseNode = copy(binding = binding)
}

// parser_rule
data class ParserRule(val name: TerminalNode, val node: Node) : BaseNode {
    override val comment: String
        get() = "${name.text}: ${node.comment}"
}

// token_rule
data class TokenRule(val name: TerminalNode, val literal: TokenLit) : BaseNode {
    override val comment: String
        get() = "${name.text}: ${literal.comment}"
}

// top_level
data class Grammar(val parserRules: List<ParserRule>, val tokenRules: List<TokenRule>)


class BuildAST : HRPGBaseVisitor<Any?>() {
    override fun visitTop_level(ctx: HRPGParser.Top_levelContext?): Any {
        val parserRules = mutableListOf<ParserRule>()
        val tokenRules = mutableListOf<TokenRule>()

        for (context in ctx?.entry() ?: listOf()) {
            when (val node = visit(context)) {
                is ParserRule -> parserRules.add(node)
                is TokenRule -> tokenRules.add(node)
                else -> throw UnsupportedOperationException("Unknown type")
            }
        }
        return Grammar(parserRules, tokenRules)
    }

    override fun visitEntry(ctx: HRPGParser.EntryContext?): Any? =
        if (ctx != null) visit(ctx.getChild(0)) else null

    override fun visitParser_rule(ctx: HRPGParser.Parser_ruleContext?): Any? =
        if (ctx != null) ParserRule(ctx.RULE_NAME(), visit(ctx.rule_body()) as Node) else null

    override fun visitRule_body(ctx: HRPGParser.Rule_bodyContext?): Any? {
        val nodes = ctx?.rule_piece()?.map { visit(it) } ?: listOf()

        return when (nodes.size) {
            0 -> null
            1 -> nodes[0]
            else -> Alternatives(null, nodes.map { it as Node })
        }
    }

    override fun visitRule_piece(ctx: HRPGParser.Rule_pieceContext?): Any? {
        val nodes = ctx?.rule_part()?.map { visit(it) } ?: listOf()
        val binding = ctx?.RULE_NAME()

        return when (nodes.size) {
            0 -> null
            1 -> (nodes[0] as Node).tryBinding(binding)
            else -> MultipartBody(binding, nodes.map { it as Node })
        }
    }

    override fun visitRule_part(ctx: HRPGParser.Rule_partContext?): Any? {
        if (ctx == null) return null

        val ruleBody = ctx.rule_body()
        if (ruleBody != null) return ZeroOrOne(null, visit(ruleBody) as Node, true)

        val node = visit(ctx.rule_elem()) as Node
        val suffixCtx = ctx.suffix()
        val suffix = if (suffixCtx != null) visit(suffixCtx) as TerminalNode? else null

        return when (suffix?.text) {
            "+" -> OneOrMore(null, node)
            "*" -> ZeroOrMore(null, node)
            "?" -> ZeroOrOne(null, node, false)
            null -> node
            else -> throw UnsupportedOperationException("Unknown suffix: ${suffix.text}")
        }
    }

    override fun visitParensRuleBody(ctx: HRPGParser.ParensRuleBodyContext?): Any? =
        if (ctx != null) visit(ctx.rule_body()) else null

    override fun visitTokRuleName(ctx: HRPGParser.TokRuleNameContext?): Any? =
        if (ctx != null) RuleRef(null, ctx.RULE_NAME()) else null

    override fun visitTokTokenName(ctx: HRPGParser.TokTokenNameContext?): Any? =
        if (ctx != null) TokenRef(null, ctx.TOKEN_NAME()) else null

    override fun visitTokTokenLit(ctx: HRPGParser.TokTokenLitContext?): Any? =
        if (ctx != null) TokenLit(null, ctx.TOKEN_LIT()) else null

    override fun visitTokPlus(ctx: HRPGParser.TokPlusContext?): Any? = ctx?.PLUS()

    override fun visitTokStar(ctx: HRPGParser.TokStarContext?): Any? = ctx?.STAR()

    override fun visitTokQuestMark(ctx: HRPGParser.TokQuestMarkContext?): Any? = ctx?.QUEST_MARK()

    override fun visitToken_rule(ctx: HRPGParser.Token_ruleContext?): Any? {
        if (ctx == null) return null

        return TokenRule(ctx.TOKEN_NAME(), TokenLit(null, ctx.TOKEN_LIT()))
    }
}
