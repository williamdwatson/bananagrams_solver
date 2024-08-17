import { useEffect, useRef, useState, MouseEvent, RefObject } from "react";
import { ConfirmDialog, confirmDialog } from "primereact/confirmdialog";
import { ContextMenu } from "primereact/contextmenu";
import { MenuItem } from "primereact/menuitem";
import { writeText } from "@tauri-apps/api/clipboard";
import { Toast } from "primereact/toast";
import html2canvas from "html2canvas";
import SolutionTime from "./solution_time";
import { result_t } from "./types";
import { TransformComponent, TransformWrapper } from "react-zoom-pan-pinch";
import WrapperControls from "./wrapper_controls";

interface ResultsDisplayProps {
    /**
     * 2D array of the results board (or an empty array if not processed)
     */
    results: result_t|null,
    /**
     * Mouse event for a right-click in the results SplitterPanel
     */
    contextMenu: MouseEvent<HTMLDivElement>|null,
    /**
     * Toast reference for displaying alerts
     */
    toast: RefObject<Toast>,
    /**
     * Function to clear the board's results
     */
    clearResults: () => void,
    /**
     * Whether the solver is running
     */
    running: boolean,
    /**
     * The width of the containing panel in percent
     */
    panelWidth: number|null
}

/**
 * Gets the current dimensions of the window; see https://stackoverflow.com/a/36862446
 * @returns The window's dimensions
 */
function getWindowDimensions() {
    const { innerWidth: width, innerHeight: height } = window;
    return {
        width,
        height
    };
}

/**
 * Hook to get the window's width and height; see https://stackoverflow.com/a/36862446
 * @returns The window's dimensions
 */
function useWindowDimensions() {
    const [windowDimensions, setWindowDimensions] = useState(getWindowDimensions());

    useEffect(() => {
        function handleResize() {
            setWindowDimensions(getWindowDimensions());
        }

        window.addEventListener('resize', handleResize);
        return () => window.removeEventListener('resize', handleResize);
    }, []);

    return windowDimensions;
}

/**
 * Displays the solution
 * 
 * @component
 */
export default function ResultsDisplay(props: ResultsDisplayProps) {
    const cm = useRef<ContextMenu|null>(null);
    const { height, width } = useWindowDimensions();

    /**
     * Copies the solution to the clipboard as text
     * 
     * @param what Whether to copy as `text` or for pasting in a `table` (like Excel);
     * the only difference is that "table" inserts tabs between each character in a row
     */
    const copyResults = (what: "text"|"table") => {
        if (props.results != null) {
            let s = '';
            props.results.board.forEach((row, i) => {
                if (row.some(val => val.trim() !== "")) {
                    row.forEach((val, j) => {
                        s += (val.trim() === "" ? " " : val) + (what === "table" && j < row.length-1 ? "\t" : "");
                    });
                }
                if (i < props.results!.board.length-2) {
                    s += "\n";
                }
            });
            writeText(s);
        }
    }

    /**
     * Downloads a data URI; see https://stackoverflow.com/a/15832662
     * @param uri Data URI to download
     * @param name Default name of the file to save
     */
    const downloadURI = (uri: string, name: string) => {
        const link = document.createElement("a");
        link.download = name;
        link.href = uri;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
    }

    /**
     * Saves the solution board as a PNG file
     */
    const saveImage = () => {
        const table = document.getElementById("results-table");
        if (table != null) {
            html2canvas(table).then(canvas => {
                const img = canvas.toDataURL("image/png");
                downloadURI(img, "Bananagrams solution.png");
            })
            .catch(error => props.toast.current?.show({ severity: "error", summary: "Failed to save image", detail: "The image failed to save: " + error}));
        }
        else {
            props.toast.current?.show({ severity: "error", summary: "Failed to save image", detail: "The image failed to save because the results object could not be located"});
        }
    }

    /**
     * Resets the board after asking for confirmation
     */
    const resetBoard = () => {
        confirmDialog({
            message: "Are you sure you want to reset the board?",
            header: "Reset?",
            icon: "pi pi-exclamation-triangle",
            accept: props.clearResults
        });
    }

    /**
     * Context menu items
     */
    const items: MenuItem[] = [
        { label: "Copy as text", icon: "pi pi-copy", disabled: props.results == null || props.results.board.length === 0, command: () => copyResults("text")},
        { label: "Copy as table", icon: "pi pi-file-excel", disabled: props.results == null || props.results.board.length === 0, command: () => copyResults("table")},
        { separator: true },
        { label: "Save as image", icon: "pi pi-save", disabled: props.results == null || props.results.board.length === 0, command: saveImage },
        { separator: true },
        { label: "Reset", icon: "pi pi-eraser", disabled: props.results == null || props.results.board.length === 0 || props.running, command: resetBoard}
    ];

    // Effect to display the custom context menu when a right-click occurs
    useEffect(() => {
        if (props.contextMenu != null) {
            cm.current?.show(props.contextMenu);
        }
    }, [props.contextMenu]);

    // Resize the results as appropriate
    useEffect(() => {
        const results_table = document.getElementById("results-table");
        if (results_table != null) {
            let n = 115;
            results_table.style.fontSize = n + "%";
            while (n >= 1 && (results_table.offsetHeight > height*0.9 || (props.panelWidth != null && (results_table.offsetWidth > 0.9*width*(props.panelWidth/100))))) {
                n -= 1;
                results_table.style.fontSize = n + "%";
            }
        }
    }, [props.results, props.panelWidth, height, width]);

    let num_letters = 0;
    props.results?.board.forEach(row => {
        row.forEach(val => {
            if (val.trim() !== "") {
                num_letters += 1;
            }
        });
    });
    let density = props.results == null ? 0 : num_letters/(props.results.board.length*props.results.board[0].length)

    return (
        <>
        <ConfirmDialog/>
        <ContextMenu model={items} ref={cm}/>
        {props.results == null || props.results.board.length === 0 ? null :
        <>
        <TransformWrapper centerOnInit onPanningStart={() => document.getElementById("results-table")!.style.cursor = "grabbing"} onPanningStop={() => document.getElementById("results-table")!.style.cursor = "grab"}>
            <WrapperControls/>
            <TransformComponent wrapperStyle={{width: "100%", height: "calc(100% - 4ch)"}}>
                <table id="results-table">
                    <tbody className="results-tbody">
                        {props.results.board.map((row, i) => {
                            return (
                                <tr key={"row-"+i} className="results-tr">
                                    {row.map((val, j) => {
                                        if (val.trim() === "") {
                                            return <td key={"row-"+i+"-cell-"+j} className="emptyCell"></td>
                                        }
                                        else if (val.length === 2) {
                                            return <td key={"row-"+i+"-cell-"+j} className="previouslyPlayedCell" aria-label={`Row ${i}, column ${j}: ${val.charAt(0)} (previously played)`}>{val.charAt(0)}</td>
                                        }
                                        else {
                                            return <td key={"row-"+i+"-cell-"+j} className="occupiedCell" aria-label={`Row ${i}, column ${j}: ${val}`}>{val}</td>
                                        }
                                    })}
                                </tr>
                            )
                        })}
                    </tbody>
                </table>
            </TransformComponent>
        </TransformWrapper>
        <SolutionTime time={props.results.elapsed} num_letters={num_letters} density={density}/>
        </>
        }
        </>
    )
}