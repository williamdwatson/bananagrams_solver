import React, { useEffect, useRef, MouseEvent, RefObject } from "react";
import { ConfirmDialog, confirmDialog } from "primereact/confirmdialog";
import { ContextMenu } from "primereact/contextmenu";
import { MenuItem } from "primereact/menuitem";
import { writeText } from "@tauri-apps/api/clipboard";
import { Toast } from "primereact/toast";
import html2canvas from "html2canvas";

interface ResultsDisplayProps {
    /**
     * 2D array of the results board (or an empty array if not processed)
     */
    results: string[][],
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
    running: boolean
}

/**
 * The results board
 * 
 * @component 
 */
export default function ResultsDisplay(props: ResultsDisplayProps) {
    const cm = useRef<ContextMenu|null>(null);

    /**
     * Copies the solution to the clipboard as text
     * 
     * @param what Whether to copy as `text` or for pasting in a `table` (like Excel);
     * the only difference is that "table" inserts tabs between each character in a row
     */
    const copyResults = (what: "text"|"table") => {
        let s = '';
        props.results.forEach((row, i) => {
            if (row.some(val => val.trim() !== "")) {
                row.forEach((val, j) => {
                    s += (val.trim() === "" ? " " : val) + (what === "table" && j < row.length-1 ? "\t" : "");
                });
            }
            if (i < props.results.length-2) {
                s += "\n";
            }
        });
        writeText(s);
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
        { label: "Copy as text", icon: "pi pi-copy", disabled: props.results.length === 0, command: () => copyResults("text")},
        { label: "Copy as table", icon: "pi pi-file-excel", disabled: props.results.length === 0, command: () => copyResults("table")},
        { separator: true },
        { label: "Save as image", icon: "pi pi-save", disabled: props.results.length === 0, command: saveImage },
        { separator: true },
        { label: "Reset", icon: "pi pi-eraser", disabled: props.results.length === 0 || props.running, command: resetBoard}
    ];

    // Effect to display the custom context menu when a right-click occurs
    useEffect(() => {
        if (props.contextMenu != null) {
            cm.current?.show(props.contextMenu);
        }
    }, [props.contextMenu]);

    return (
        <>
        <ConfirmDialog/>
        <ContextMenu model={items} ref={cm}/>
        {props.results.length === 0 ? null :
        <table id="results-table">
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