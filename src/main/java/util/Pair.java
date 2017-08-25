package util;

public final class Pair<T, U> {
    private final T first;
    private final U second;

    public Pair(T first, U second) {
        this.first = first;
        this.second = second;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;

        Pair<?, ?> pair = (Pair<?, ?>) o;

        if (!getFirst().equals(pair.getFirst())) return false;
        return getSecond().equals(pair.getSecond());
    }

    @Override
    public int hashCode() {
        int result = getFirst().hashCode();
        result = 31 * result + getSecond().hashCode();
        return result;
    }

    public T getFirst() {
        return first;
    }

    public U getSecond() {
        return second;
    }
}
