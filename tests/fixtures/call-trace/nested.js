// Chain: a -> b -> c
function a() {
    b();
}

function b() {
    c();
}

function c() {
    console.log("leaf");
}

// Cycle: cycleA -> cycleB -> cycleA
function cycleA() {
    cycleB();
}

function cycleB() {
    cycleA();
}

// Multiple callers: caller1 -> target, caller2 -> target
function target() {
    console.log("target");
}

function caller1() {
    target();
}

function caller2() {
    target();
}
