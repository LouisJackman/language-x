package util;

import junit.framework.TestCase;

import java.util.List;
import java.util.Optional;

import static java.util.Arrays.asList;
import static java.util.Collections.*;
import static java.util.Optional.empty;

public class ListsTest extends TestCase {
    private static final List<Integer> xs = unmodifiableList(asList(1, 3, 5, 7));

    public void testHead() {
        assertEquals(Optional.of(1), Lists.head(xs));
        assertEquals(empty(), Lists.head(emptyList()));
    }

    public void testTail() {
        assertEquals(Optional.of(asList(3, 5, 7)), Lists.tail(xs));
        assertEquals(empty(), Lists.tail(emptyList()));
    }

    public void testCons() {
        assertEquals(asList(10, 1, 3, 5, 7), Lists.cons(10, xs));
        assertEquals(singletonList('A'), Lists.cons('A', emptyList()));
    }
}
