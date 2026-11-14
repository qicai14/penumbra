/** Wraps an async function so calls run one at a time, in call order. A rejection goes to its own caller only; the next call still runs. */
export function serialize<A extends unknown[], R>(fn: (...args: A) => Promise<R>): (...args: A) => Promise<R> {
  let chain: Promise<unknown> = Promise.resolve();
  return (...args) => {
    const run = chain.then(() => fn(...args));
    chain = run.catch(() => {});
    return run;
  };
}
