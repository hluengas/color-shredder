/* tslint:disable */
/* eslint-disable */

export class ShredderEngine {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Total byte length of the linear RGBA pixel buffer (width * height * 4).
     */
    get_rgba_len(): number;
    /**
     * Pointer to the linear RGBA pixel byte buffer in WebAssembly linear memory.
     */
    get_rgba_ptr(): number;
    is_finished(): boolean;
    constructor(width: number, height: number, start_x: number, start_y: number, strategy_num: number, bit_depth: number, color_space_str: string, fixed_channel_opt: number | null | undefined, shuffle: boolean, seed_opt?: number | null);
    /**
     * Advance the simulation by up to `batch_size` pixels.
     * Internal candidate search utilizes Rayon multithreading across Web Workers.
     */
    step(batch_size: number): WasmStats;
}

export class WasmStats {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    is_finished: boolean;
    percent_complete: number;
    pixels_available: number;
    pixels_placed: number;
    total_pixels: number;
}

export function initThreadPool(num_threads: number): Promise<any>;

export class wbg_rayon_PoolBuilder {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    build(): void;
    numThreads(): number;
    receiver(): number;
}

export function wbg_rayon_start_worker(receiver: number): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_get_wasmstats_is_finished: (a: number) => number;
    readonly __wbg_get_wasmstats_percent_complete: (a: number) => number;
    readonly __wbg_get_wasmstats_pixels_available: (a: number) => number;
    readonly __wbg_get_wasmstats_pixels_placed: (a: number) => number;
    readonly __wbg_get_wasmstats_total_pixels: (a: number) => number;
    readonly __wbg_set_wasmstats_is_finished: (a: number, b: number) => void;
    readonly __wbg_set_wasmstats_percent_complete: (a: number, b: number) => void;
    readonly __wbg_set_wasmstats_pixels_available: (a: number, b: number) => void;
    readonly __wbg_set_wasmstats_pixels_placed: (a: number, b: number) => void;
    readonly __wbg_set_wasmstats_total_pixels: (a: number, b: number) => void;
    readonly __wbg_shredderengine_free: (a: number, b: number) => void;
    readonly __wbg_wasmstats_free: (a: number, b: number) => void;
    readonly __wbg_wbg_rayon_poolbuilder_free: (a: number, b: number) => void;
    readonly initThreadPool: (a: number) => any;
    readonly shredderengine_get_rgba_len: (a: number) => number;
    readonly shredderengine_get_rgba_ptr: (a: number) => number;
    readonly shredderengine_is_finished: (a: number) => number;
    readonly shredderengine_new: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => number;
    readonly shredderengine_step: (a: number, b: number) => number;
    readonly wbg_rayon_poolbuilder_build: (a: number) => void;
    readonly wbg_rayon_poolbuilder_numThreads: (a: number) => number;
    readonly wbg_rayon_poolbuilder_receiver: (a: number) => number;
    readonly wbg_rayon_start_worker: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
