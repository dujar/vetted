/**
 * Append point: later steps add module files here and re-export them —
 * never edit another step's module (wire.md conventions).
 */
export * from "./abi";
export * from "./events";
export * from "./types";
export * from "./watchdog";
// events.ts ships with step-3's merge — its re-export lands there, not here.
