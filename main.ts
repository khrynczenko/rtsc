function assert(x: boolean) {
    if (x) {
        putchar(46);
    } else {
        putchar(70);
    }
}

function assert_four_parameters_work(x: number, y: number, z: number, w: number) {
    assert(x == 1);
    assert(y == 2);
    assert(z == 3);
    assert(w == 4);
}

function add_one(x: number) {
    return x + 1;
}

function factorial(n: number) {
    if (n == 0) {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

function fibonacci(nth: number) {
    if (nth == 0) {
        return 0;
    } else {
        if (nth == 1) {
            return 1;
        } else {
            return fibonacci(nth - 1) + fibonacci(nth - 2);
        }
    }
}


function main() {
    assert(true);
    assert(42 == 4 + 2 * (12 - 2) + 3 * (5 + 1));

    if (true) {
        assert(true);
    } else {
        assert(false);
    }

    assert_four_parameters_work(1, 2, 3, 4);
    assert(add_one(1) == 2);
    assert(factorial(5) == 120); // 5 * 4 * 3 * 2 * 1 * 1;
    assert(fibonacci(3) == 2);

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

    var a = [10, 20, 30];
    assert(a[0] == 10);
    assert(a[1] == 20);
    assert(a[2] == 30);
    assert(a[3] == undefined);   // Bounds checking.
    assert(length(a) == 3);
}
