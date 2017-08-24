package util;

import junit.framework.TestCase;

import java.util.*;

public class CollectionsTest extends TestCase {

    public void testUpdateSet() {
        Set<Boolean> xs = new HashSet<>();
        Set<Boolean> ys = new HashSet<>();
        xs.add(true);
        ys.add(false);
        Set<Boolean> zs = Collections.update(xs, ys);

        assertTrue(zs.contains(true));
        assertTrue(zs.contains(false));
    }

    public void testUpdateList() {
        List<Integer> xs = new ArrayList<>();
        List<Integer> ys = new ArrayList<>();
        xs.add(1);
        ys.add(42);
        ys.add(10000);
        List<Integer> zs = Collections.update(xs, ys);

        assertTrue(zs.contains(1));
        assertTrue(zs.contains(42));
        assertTrue(zs.contains(10000));
    }

    public void testUpdateMap() {
        Map<Integer, Integer> xs = new HashMap<>();
        Map<Integer, Integer> ys = new HashMap<>();
        xs.put(1, 12);
        ys.put(42, 32);
        ys.put(10000, 99);
        Map<Integer, Integer> zs = Collections.update(xs, ys);

        assertEquals(12, (int) zs.get(1));
        assertEquals(32, (int) zs.get(42));
        assertEquals(99, (int) zs.get(10000));
    }
}
