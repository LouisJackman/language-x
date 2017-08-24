import lexing.Lexer;
import lexing.SourceFileStream;
import lexing.Token;
import util.UncheckedException;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

import static java.lang.System.err;
import static java.lang.System.out;

/**
 * The main application.
 */
final class Main {

    /**
     * Runs the language file passed in as the argument.
     */
    public static void main(String[] args) throws IOException {
        if (args.length == 1) {
            Path path = Paths.get(args[0]);
            if (!Files.exists(path)) {
                abort("path does not exist '%s'", path.toString());
            } else if (!Files.isRegularFile(path)) {
                abort("file is not a regular file: '%s'", path.toString());
            } else if (!Files.isReadable(path)) {
                abort("file is not readable: '%s'", path.toString());
            } else {
                try {
                    run(path);
                } catch (Exception e) {
                    abort("An error occured: %s", e.getMessage());
                }
            }
        } else {
            displayUsage();
        }
    }

    private static void abort(String format, Object... args) {
        String message = String.format(format, args);
        err.println("ERROR: " + message);
        System.exit(1);
    }

    private static void displayUsage() {
        abort("usage: java language-x/Main.class file-to-run");
    }

    private static void run(Path path) {
        try (InputStream reader = Files.newInputStream(path)) {
            Lexer lexer = new Lexer(new SourceFileStream(reader));
            while (lexer.hasNext()) {
                Token<?> token = lexer.next();
                out.println(token);
            }
        } catch (IOException e) {
            throw new UncheckedException(e);
        }
    }
}
