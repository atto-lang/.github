// Bounded, strict object-member audit before JSON.parse.
// JSON.parse on its own silently accepts conflicting duplicate members.
export function parseSnapshotJSON(source) {
  if (typeof source !== 'string' || source.length > 256 * 1024) throw Error('UNSAFE_JSON_SIZE');
  let index = 0, root = 'value', totalKeys = 0;
  const stack = [];
  const reject = () => { throw Error('UNSAFE_JSON_STRUCTURE'); };
  const markValue = () => {
    if (stack.length === 0) {
      if (root !== 'value') reject();
      root = 'done';
    } else {
      const frame = stack[stack.length - 1];
      if (frame.kind === 'object') {
        if (frame.state !== 'value') reject();
      } else if (frame.state !== 'valueOrEnd' && frame.state !== 'valueRequired') reject();
      frame.state = 'commaOrEnd';
    }
  };
  const stringToken = () => {
    const start = index++;
    let ended = false;
    while (index < source.length) {
      const ch = source[index++];
      if (ch === '"') { ended = true; break; }
      if (ch === '\\') {
        if (index >= source.length) reject();
        index++;
      }
    }
    if (!ended) reject();
    return JSON.parse(source.slice(start, index));
  };
  const markClose = (kind) => {
    const frame = stack[stack.length - 1];
    if (!frame || frame.kind !== kind ||
        !['keyOrEnd', 'valueOrEnd', 'commaOrEnd'].includes(frame.state)) reject();
    stack.pop();
    index++;
  };
  while (index < source.length) {
    const ch = source[index];
    if (ch === ' ' || ch === '\t' || ch === '\n' || ch === '\r') { index++; continue; }
    const frame = stack[stack.length - 1];
    if (ch === '"') {
      if (frame?.kind === 'object' && (frame.state === 'keyOrEnd' || frame.state === 'keyRequired')) {
        const key = stringToken();
        if (++totalKeys > 8192 || frame.keys.has(key) ||
            key === '__proto__' || key === 'constructor' || key === 'prototype') reject();
        frame.keys.add(key);
        frame.state = 'colon';
      } else { markValue(); stringToken(); }
    } else if (ch === '{' || ch === '[') {
      markValue();
      if (stack.length >= 64) reject();
      stack.push(ch === '{' ? {kind:'object', state:'keyOrEnd', keys:new Set()} : {kind:'array', state:'valueOrEnd'});
      index++;
    } else if (ch === '}') markClose('object');
    else if (ch === ']') markClose('array');
    else if (ch === ':') {
      if (!frame || frame.kind !== 'object' || frame.state !== 'colon') reject();
      frame.state = 'value'; index++;
    } else if (ch === ',') {
      if (!frame || frame.state !== 'commaOrEnd') reject();
      frame.state = frame.kind === 'object' ? 'keyRequired' : 'valueRequired'; index++;
    } else {
      markValue();
      const start = index;
      while (index < source.length && !',}] \t\n\r'.includes(source[index])) index++;
      if (start === index) reject();
    }
  }
  if (stack.length || root !== 'done') reject();
  return JSON.parse(source);
}
