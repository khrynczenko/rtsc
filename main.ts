function assert(x) {
    if (x) {
        putchar(46);
    } else {
        putchar(70);
    }
}

function assert_four_parameters_work(x, y, z, w) {
    assert(x == 1);
    assert(y == 2);
    assert(z == 3);
    assert(w == 4);
}

function add_one(x) {
    return x + 1;
}

function factorial(n) {
    if (n == 0) {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

function main() {
    assert(true);
    assert(1);
    assert(!0);
    assert(42 == 4 + 2 * (12 - 2) + 3 * (5 + 1));
    assert(rand() != 42);

    if (1) {
        assert(1);
    } else {
        assert(0);
    }

    if (0) {
        assert(0);
    } else {
        assert(1);
    }

    assert_four_parameters_work(1, 2, 3, 4);
    assert(add_one(1) == 2);
    assert(factorial(5) == 120); // 5 * 4 * 3 * 2 * 1 * 1;

    var a = 1;
    assert(a == 1);
    a = 0;
    assert(a == 0);

    var i = 4;
    while (i != 4) {
        i = i + 1;
    }
    assert(i == 4);

    assert(true);
    assert(!false);
    assert(!undefined);
    assert(!null);

    var a = [10, 20, 30];
    assert(a[0] == 10);
    assert(a[1] == 20);
    assert(a[2] == 30);
    assert(a[3] == undefined);  // Bounds checking.
    assert(length(a) == 3);
}
