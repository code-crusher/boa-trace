assertThrows(() => {
  structuredClone(() => {});
});

{
  const original = new Error("boom");
  const cloned = structuredClone(original);

  assert(cloned instanceof Error);
  assertNEq(cloned, original);
  assertEq(cloned.name, "Error");
  assertEq(cloned.message, "boom");
  assertEq(cloned.stack, original.stack);
}

{
  const original = new TypeError("bad");
  const cloned = structuredClone(original);

  assert(cloned instanceof TypeError);
  assertEq(cloned.name, "TypeError");
  assertEq(cloned.message, "bad");
}

{
  const original = new AggregateError([new Error("inner")], "agg");
  const cloned = structuredClone(original);

  assert(cloned instanceof AggregateError);
  assertEq(cloned.message, "agg");
  assertArrayEqual(cloned.errors.map((e) => e.message), ["inner"]);
}

{
  const original = new Error("boom", { cause: { code: 7 } });
  const cloned = structuredClone(original);

  assertNEq(cloned.cause, original.cause);
  assertEq(cloned.cause.code, 7);
}

{
  const original = new Error("boom", { cause: undefined });
  const cloned = structuredClone(original);

  assert(Object.hasOwn(cloned, "cause"));
  assertEq(cloned.cause, undefined);
}

{
  const original = new Error("boom");
  original.cause = original;

  const cloned = structuredClone(original);
  assertNEq(cloned, original);
  assertEq(cloned.cause, cloned);
}

{
  const original = new Error("boom");
  original.name = "CustomError";

  const cloned = structuredClone(original);
  assert(cloned instanceof Error);
  assertEq(cloned.name, "CustomError");
}
