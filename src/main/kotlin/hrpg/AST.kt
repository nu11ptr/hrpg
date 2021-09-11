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
    override fun visitTopLevel(ctx: HRPGParser.TopLevelContext): Any {
        val parserRules = mutableListOf<ParserRule>()
        val tokenRules = mutableListOf<TokenRule>()

        for (context in ctx.entry()) {
            when (val node = visit(context)) {
                is ParserRule -> parserRules.add(node)
                is TokenRule -> tokenRules.add(node)
                else -> throw UnsupportedOperationException("Unknown type")
            }
        }
        return Grammar(parserRules, tokenRules)
    }

    override fun visitEntry(ctx: HRPGParser.EntryContext): Any? = visit(ctx.getChild(0))

    override fun visitParseRule(ctx: HRPGParser.ParseRuleContext): Any =
        ParserRule(ctx.RULE_NAME(), visit(ctx.ruleBody()) as Node)

    override fun visitRuleBody(ctx: HRPGParser.RuleBodyContext): Any? {
        val nodes = ctx.rulePiece().map { visit(it) }

        return when (nodes.size) {
            1 -> nodes[0]
            else -> Alternatives(null, nodes.map { it as Node })
        }
    }

    override fun visitRulePiece(ctx: HRPGParser.RulePieceContext): Any? {
        val nodes = ctx.rulePart().map { visit(it) }
        val binding = ctx.RULE_NAME()

        return when (nodes.size) {
            0 -> null
            1 -> (nodes[0] as Node).tryBinding(binding)
            else -> MultipartBody(binding, nodes.map { it as Node })
        }
    }

    override fun visitRulePart(ctx: HRPGParser.RulePartContext): Any {
        val ruleBody = ctx.ruleBody()
        if (ruleBody != null) return ZeroOrOne(null, visit(ruleBody) as Node, true)

        val node = visit(ctx.ruleElem()) as Node
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

    override fun visitParensRuleBody(ctx: HRPGParser.ParensRuleBodyContext): Any? =
        visit(ctx.ruleBody())

    override fun visitTokRuleName(ctx: HRPGParser.TokRuleNameContext): Any =
        RuleRef(null, ctx.RULE_NAME())

    override fun visitTokTokenName(ctx: HRPGParser.TokTokenNameContext): Any =
        TokenRef(null, ctx.TOKEN_NAME())

    override fun visitTokTokenLit(ctx: HRPGParser.TokTokenLitContext): Any =
        TokenLit(null, ctx.TOKEN_LIT())

    override fun visitTokPlus(ctx: HRPGParser.TokPlusContext): Any? = ctx.PLUS()

    override fun visitTokStar(ctx: HRPGParser.TokStarContext): Any? = ctx.STAR()

    override fun visitTokQuestMark(ctx: HRPGParser.TokQuestMarkContext): Any? = ctx.QUEST_MARK()

    override fun visitTokenRule(ctx: HRPGParser.TokenRuleContext): Any =
        TokenRule(ctx.TOKEN_NAME(), TokenLit(null, ctx.TOKEN_LIT()))
}
