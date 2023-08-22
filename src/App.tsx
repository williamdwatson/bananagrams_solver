import { useState, useRef } from "react";
import "primereact/resources/themes/tailwind-light/theme.css";
import "primereact/resources/primereact.min.css";
import 'primeicons/primeicons.css';
import { Splitter, SplitterPanel } from "primereact/splitter";
import { Toast } from "primereact/toast";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import LetterInput from "./letter_input";
import ResultsDisplay from "./results_display";

function App() {
    const toast = useRef<Toast>(null);
    const [running, setRunning] = useState(false);
    const [results, setResults] = useState<string[][]>([]);

    const startRunning = (letters: Map<string, number>) => {
        setRunning(true);
        invoke("play_bananagrams", { availableLetters: letters })
            .then(res => {
                console.log(res);
                setResults(res as string[][]);
            })
            .catch(error => {
                toast.current?.show({severity: "error", summary: "Uh oh!", detail: "" + error});
            })
            .finally(() => setRunning(false));
    }    

    return (
        <>
        <Toast ref={toast}/>
        <Splitter style={{height: "98vh"}}>
            <SplitterPanel size={20}>
                <LetterInput toast={toast} startRunning={startRunning} running={running}/>
            </SplitterPanel>
            <SplitterPanel size={80} style={{display: "flex", justifyContent: "center", alignItems: "center"}}>
                <ResultsDisplay results={results}/>
            </SplitterPanel>
        </Splitter>
        </>
    );
}

export default App;
