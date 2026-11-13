// https://github.com/boa-dev/boa/issues/5530

{
  // Cloning a `DataView` preserves its offset and length, and clones the
  // underlying buffer instead of sharing it.
  const buffer = new ArrayBuffer(16);
  const view = new DataView(buffer, 4, 8);
  view.setInt32(0, 42);

  const copy = structuredClone(view);

  assertNEq(copy, view);
  assert(copy instanceof DataView);
  assertEq(copy.byteOffset, 4);
  assertEq(copy.byteLength, 8);
  assertEq(copy.getInt32(0), 42);
  assertNEq(copy.buffer, buffer);
  assertEq(copy.buffer.byteLength, 16);

  // The clone doesn't share the buffer with the original.
  view.setInt32(0, 7);
  assertEq(copy.getInt32(0), 42);

  copy.setInt32(0, 13);
  assertEq(view.getInt32(0), 7);
}

{
  // A `DataView` nested inside another object is cloned too.
  const buffer = new ArrayBuffer(8);
  const original = {
    view: new DataView(buffer, 2, 4),
    buffer,
  };
  original.view.setUint8(0, 0xab);

  const dolly = structuredClone(original);

  assertEq(dolly.view.byteOffset, 2);
  assertEq(dolly.view.byteLength, 4);
  assertEq(dolly.view.getUint8(0), 0xab);
  assertNEq(dolly.view.buffer, buffer);
  assertEq(dolly.view.buffer, dolly.buffer);
}
