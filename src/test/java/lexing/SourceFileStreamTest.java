package lexing;

import junit.framework.TestCase;

import java.io.ByteArrayInputStream;
import java.nio.charset.StandardCharsets;

public final class SourceFileStreamTest extends TestCase {

    public void test() {
        byte[] mockInput = "abcdefghi".getBytes(StandardCharsets.UTF_8);
        SourceFileStream stream = new SourceFileStream(new ByteArrayInputStream(mockInput));

        assertEquals("abcde", stream.lookAhead(5).get());
        assertEquals("abc", stream.read(3).get());
        assertEquals("defg", stream.lookAhead(4).get());
    }
}
