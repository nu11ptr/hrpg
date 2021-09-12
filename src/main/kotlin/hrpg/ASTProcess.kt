package hrpg

import org.antlr.v4.runtime.tree.TerminalNode

typealias Literals = MutableMap<String, Pair<TerminalNode, TokenRef?>>

const val EOF = "EOF"
const val ILLEGAL = "ILLEGAL"

class ASTProcess(private val grammar: Grammar) {
    private val tokenNames = mutableSetOf(EOF, ILLEGAL)
    private val literals: Literals = mutableMapOf()
    private val errors = mutableListOf<String>()

    fun process(): Triple<Grammar, Set<String>, List<String>> {
        val tokenRules = grammar.tokenRules.map { processTokenRule(it) }
        val parseRules = grammar.parserRules.map { processParserRule(it) }
        return Triple(Grammar(parseRules, tokenRules), tokenNames, errors)
    }

    private fun logError(msg: String) {
        errors.add("ERROR: $msg")
    }

    private fun processTokenRule(rule: TokenRule): TokenRule {
        // Strip quotes and store processed string as literal key
        val rawLit = rule.literal.literal.text
        val lit = rawLit.subSequence(1, rawLit.length - 1).toString()
        literals[lit] = Pair(rule.name, null)

        tokenNames.add(rule.name.text)
        return rule
    }

    private fun processParserRule(rule: ParserRule): ParserRule {
        val body = processNode(rule.node)
        return if (body === rule.node) rule else ParserRule(rule.name, body)
    }

    private fun processNode(node: Node, parent: NodeContainer? = null): Node {
        if (parent == null && node.binding != null)
            logError("Top level binding '${node.binding!!.text}' is not allowed")

        return when (node) {
            is Alternatives -> processAlternatives(node)
            is MultipartBody -> processMultipartBody(node)
            is ZeroOrMore -> processZeroOrMore(node)
            is OneOrMore -> processOneOrMore(node)
            is ZeroOrOne -> processZeroOrOne(node)
            is RuleRef -> processRuleRef(node)
            is TokenRef -> processTokenRef(node)
            is TokenLit -> processTokenLit(node)
        }
    }

    private fun processAlternatives(node: Alternatives): Node =
        Alternatives(node.binding, node.nodes.map { processNode(it, node) })

    private fun processMultipartBody(node: MultipartBody): Node =
        MultipartBody(node.binding, node.nodes.map { processNode(it, node) })

    private fun processZeroOrMore(node: ZeroOrMore): Node =
        ZeroOrMore(node.binding, processNode(node.node, node))

    private fun processOneOrMore(node: OneOrMore): Node =
        OneOrMore(node.binding, processNode(node.node, node))

    private fun processZeroOrOne(node: ZeroOrOne): Node =
        ZeroOrOne(node.binding, processNode(node.node, node), node.brackets)

    private fun processRuleRef(node: RuleRef): Node = node

    private fun processTokenRef(node: TokenRef): Node {
        tokenNames.add(node.name.text)
        return node
    }

    private fun processTokenLit(node: TokenLit): Node {
        // Strip quotes and store processed string as literal key
        val rawLit = node.literal.text
        val lit = rawLit.subSequence(1, rawLit.length - 1).toString()

        // Try and find the literal to ensure it has a corresponding rule
        val pair = literals[lit]
        if (pair == null) {
            logError("Literal ${node.literal.text} does not have corresponding token rule")
            return node
        }

        // If TokenRef not already created, do so. Regardless, replace the lit with token ref
        var (name, ref) = pair
        if (ref == null) {
            ref = TokenRef(node.binding, name, node.literal)
            literals[lit] = Pair(name, ref)
        }

        return ref
    }
}
