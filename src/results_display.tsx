interface ResultsDisplayProps {
    /**
     * 2D array of the results board (or an empty array if not processed)
     */
    results: string[][]
}

/**
 * Displays the board solution
 * 
 * @component
 */
export default function ResultsDisplay(props: ResultsDisplayProps) {
    return (
        <>
        {props.results.length === 0 ? null :
        <table>
            <tbody>
                {props.results.map((row, i) => {
                    return (
                        <tr key={"row-"+i}>
                            {row.map((val, j) => {
                                if (val.trim() === "") {
                                    return <td key={"row-"+i+"-cell-"+j} className="emptyCell"></td>
                                }
                                else {
                                    return <td key={"row-"+i+"-cell-"+j} className="occupiedCell">{val}</td>
                                }
                            })}
                        </tr>
                    )
                })}
            </tbody>
        </table>
        }
        </>
    )
}