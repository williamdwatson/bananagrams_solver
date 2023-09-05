/**
 * Type of the return after a solution is found
 */
export type result_t = {
    /**
     * 2D array of characters of the solution
     */
    board: string[][],
    /**
     * The time the function took to run
     */
    elapsed: number
};