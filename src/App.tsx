import { useState, useRef, useEffect, MouseEvent } from "react";
import "primereact/resources/themes/tailwind-light/theme.css";
import "primereact/resources/primereact.min.css";
import 'primeicons/primeicons.css';
import { Splitter, SplitterPanel } from "primereact/splitter";
import { Toast } from "primereact/toast";
import { invoke } from "@tauri-apps/api/tauri";
import { sendNotification } from "@tauri-apps/api/notification";
import "./App.css";
import LetterInput from "./letter_input";
import ResultsDisplay from "./results_display";
import PlayableWords from "./playable_words";
import { result_t } from "./types";

export default function App() {
    const toast = useRef<Toast>(null);
    const [running, setRunning] = useState(false);
    const [results, setResults] = useState<result_t|null>(null);
    const [letterInputContextMenu, setLetterInputContextMenu] = useState<MouseEvent<HTMLDivElement>|null>(null);
    const [resultsContextMenu, setResultsContextMenu] = useState<MouseEvent<HTMLDivElement>|null>(null);
    const [playableWordsVisible, setPlayableWordsVisible] = useState(false);
    const [playableWords, setPlayableWords] = useState<{short: string[], long: string[]}|null>(null);
    const [panelSizes, setPanelSizes] = useState<number[]>([25, 75]);

    // Disable right-clicking elsewhere on the page
    useEffect(() => {
        document.addEventListener("contextmenu", e => e.preventDefault());
    }, []);

    /**
     * Runs the solver
     * @param letters Mapping of length-one letter strings to the number of that letter present in the hand
     */
    const startRunning = (letters: Map<string, number>) => {
        setRunning(true);
        invoke("play_bananagrams", { availableLetters: letters })
            .then(res => {
                const results = res as result_t;
                setResults(results);
                if (results.elapsed > 5000) {
                    sendNotification({ title: "Completed", body: "The board has been solved!" });
                }
            })
            .catch(error => {
                toast.current?.show({severity: "error", summary: "Uh oh!", detail: "" + error});
            })
            .finally(() => setRunning(false));
    }

    /**
     * Clears the existing results, if any (only if the solver is not currently running)
     */
    const clearResults = () => {
        if (!running) {
            invoke("reset").then(()=> {
                setResults(null);
            })
            .catch(error => {
                toast.current?.show({severity: "error", summary: "Uh oh!", detail: "" + error});
            });
        }
    }

    return (
        <>
        <Toast ref={toast}/>
        <PlayableWords playableWords={playableWords} visible={playableWordsVisible} setVisible={setPlayableWordsVisible}/>
        <Splitter style={{height: "98vh"}} onResizeEnd={e => setPanelSizes(e.sizes)}>
            <SplitterPanel size={panelSizes[0]} pt={{root: {onContextMenu: e => setLetterInputContextMenu(e)}}}>
                <LetterInput toast={toast} startRunning={startRunning} running={running} contextMenu={letterInputContextMenu} setPlayableWords={setPlayableWords} setPlayableWordsVisible={setPlayableWordsVisible} clearResults={clearResults}/>
            </SplitterPanel>
            <SplitterPanel size={panelSizes[1]} style={{display: "flex", justifyContent: "center", alignItems: "center"}} pt={{root: {onContextMenu: e => setResultsContextMenu(e)}}}>
                <ResultsDisplay toast={toast} results={results} contextMenu={resultsContextMenu} clearResults={clearResults} running={running} panelWidth={panelSizes[1]}/>
            </SplitterPanel>
        </Splitter>
        </>
    );
}
