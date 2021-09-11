package hrpg

import hrpg.antlr.HRPGLexer
import hrpg.antlr.HRPGParser
import org.antlr.v4.runtime.CharStreams
import org.antlr.v4.runtime.CommonTokenStream
import picocli.CommandLine
import java.io.File
import java.nio.file.Files
import java.util.concurrent.Callable
import kotlin.system.exitProcess

@CommandLine.Command(
    name = "hrpg", mixinStandardHelpOptions = true, version = ["1.0-SNAPSHOT"],
    description = ["Human Readable Parser Generator - generate parsers that look like they were written by hand"]
)
class Checksum : Callable<Int> {

    @CommandLine.Parameters(index = "0", description = ["The grammar file to parse"], paramLabel = "<grammar.hrpg>")
    lateinit var grammar: File

    @CommandLine.Option(
        names = ["-c", "--config"],
        description = ["Configuration file specifying overrides to the default configuration"],
        paramLabel = "<config.json>"
    )
    var config: File? = null

    override fun call(): Int {
        val grammarData = Files.readString(grammar.toPath())
        val configData = if (config != null) Files.readString(config!!.toPath()) else ""

        val lexer = HRPGLexer(CharStreams.fromString(grammarData))
        val tokens = CommonTokenStream(lexer)
        val parser = HRPGParser(tokens)
        val tree = parser.top_level()

        val builder = BuildAST()
        val ast = builder.visitTop_level(tree)

        println(ast)
        return 0
    }
}

fun main(args: Array<String>): Unit = exitProcess(CommandLine(Checksum()).execute(*args))
