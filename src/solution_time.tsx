import prettyMilliseconds from 'pretty-ms';

interface SolutionTimeProps {
    /**
     * The time in milliseconds it took to solve the board
     */
    time: number
}

/**
 * Displays how long it took to solve the board
 * 
 * @component
 */
export default function SolutionTime(props: SolutionTimeProps) {
    // Green-to-red color palette; see https://color-hex.org/color-palettes/187
    const COLORS = ["#2CBA00", "#8EDE00", "#D9D009", "#FFA700", "#FF0000"];
    const lims = [100, 500, 1000, 2000];
    let color = COLORS[COLORS.length-1];
    for (let i=0; i<lims.length; i++) {
        if (props.time < lims[i]) {
            color = COLORS[i];
            break;
        }
    }
    return (
        <span style={{position: "fixed", bottom: "3vh", right: "3vw"}}>
            Completed in <span style={{color: color}}>{prettyMilliseconds(props.time)}</span>
        </span>
    )
}