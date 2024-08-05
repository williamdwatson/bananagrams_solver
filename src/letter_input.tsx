import { useEffect, useRef, useState, MouseEvent, RefObject, ReactElement } from "react";
import { Button } from "primereact/button";
import { confirmDialog } from "primereact/confirmdialog";
import { ContextMenu } from "primereact/contextmenu";
import { Dialog } from "primereact/dialog";
import { InputNumber, InputNumberValueChangeEvent } from "primereact/inputnumber";
import { InputText } from "primereact/inputtext";
import { MenuItem } from "primereact/menuitem";
import { Toast } from "primereact/toast";
import { readText, writeText } from "@tauri-apps/api/clipboard";
import { invoke } from "@tauri-apps/api/tauri";
import { Dropdown } from "primereact/dropdown";
import { TabPanel, TabView } from "primereact/tabview";
import { result_t } from "./types";

interface LetterInputProps {
    /**
     * Toast reference for displaying alerts
     */
    toast: RefObject<Toast>,
    /**
     * Function to start solving the game
     * @param letters Map of every letter to the number present in the hand
     */
    startRunning: (letters: Map<string, number>) => void,
    /**
     * Whether the game is being solved or not
     */
    running: boolean,
    /**
     * Mouse event for a right-click in the letter input SplitterPanel
     */
    contextMenu: MouseEvent<HTMLDivElement>|null,
    /**
     * Sets the list of words that can be played given the tiles in the hand
     * @param words Which words can be played
     */
    setPlayableWords: (words: {short: string[], long: string[]}) => void,
    /**
     * Sets whether the visible words popup should be visible
     * @param visible Whether the popup should be visible
     */
    setPlayableWordsVisible: (visible: boolean) => void,
    /**
     * Function to clear the board's results
     */
    clearResults: () => void,
    /**
     * Whether an undo can be performed
     */
    undoPossible: boolean,
    /**
     * Whether a redo can be performed
     */
    redoPossible: boolean,
    /**
     * Sets whether an undo can be performed
     */
    setUndoPossible: (possible: boolean) => void,
    /**
     * Sets whether a redo can be performed
     */
    setRedoPossible: (possible: boolean) => void,
    /**
     * Sets the results (for undoing and redoing)
     * @param results The undone/redone results
     */
    setResults: (results: result_t|null) => void
}

/**
 * Array of all uppercase Latin letters in alphabetical order
 */
const UPPERCASE = [..."ABCDEFGHIJKLMNOPQRSTUVWXYZ"];
/**
 * Array of all digits
 */
const DIGITS = [..."0123456789"];

/**
 * For inputting a hand of letters and starting a solve
 * 
 * @component
 */
export default function LetterInput(props: LetterInputProps){
    const cm = useRef<ContextMenu|null>(null);
    const type_in_cm = useRef<ContextMenu|null>(null);
    const random_num_cm = useRef<ContextMenu|null>(null);
    const m = new Map();
    const num_letters = new Map<string, number>();
    const invalid = new Map<string, boolean>();
    const how_many = [13, 3, 3, 6, 18, 3, 4, 3, 12, 2, 2, 5, 3, 8, 11, 3, 2, 9, 6, 9, 6, 3, 3, 2, 3, 2];
    const individual_cm_refs: RefObject<ContextMenu>[] = [];
    const individual_cms: ReactElement<ContextMenu>[] = [];
    UPPERCASE.forEach((c, i) => {
        m.set(c, 0);
        num_letters.set(c, how_many[i]);
        invalid.set(c, false);
        const individual_cm = useRef<ContextMenu|null>(null);
        individual_cm_refs.push(individual_cm);
        const individual_items: MenuItem[] = [
            { label: "Copy", icon: "pi pi-copy", command: () => writeText(String(letterNums.get(c)) ?? "0")},
            { label: "Paste", icon: "pi pi-file-import", command: () => {
                readText().then(val => {
                    if (val != null && !isNaN(parseInt(val))) {
                        const new_map = new Map(letterNums);
                        new_map.set(c, parseInt(val));
                        setLetterNums(new_map);
                    }
                });
            }}
        ];
        individual_cms.push(<ContextMenu model={individual_items} ref={individual_cm} key={"cm-"+c}/>);
    });
    const [letterNums, setLetterNums] = useState<Map<string, number|null|undefined>>(m);
    const [lettersInvalid, setLettersInvalid] = useState<Map<string, boolean>>(invalid);
    const [typeInVisible, setTypeInVisible] = useState(false);
    const [typedIn, setTypedIn] = useState("");
    const [randomNum, setRandomNum] = useState("21");
    const [randomFrom, setRandomFrom] = useState<"standard Bananagrams"|"double Bananagrams"|"infinite set">("standard Bananagrams");
    const [playableWordsLoading, setPlayableWordsLoading] = useState(false);

    // Show the custom context menu on right click
    useEffect(() => {
        if (props.contextMenu != null) {
            cm.current?.show(props.contextMenu);
        }
    }, [props.contextMenu]);

    // Give focus to the text input (since there's an animation delay on it appearing, so not using a timeout doesn't always work)
    useEffect(() => {
        if (typeInVisible) {
            setTimeout(() => document.getElementById("typeIn")?.focus(), 100);
        }
    }, [typeInVisible]);

    /**
     * Copies the current hand of letters
     * @param what Whether to copy in the same format as typing in the letters (`text`), as a `table` suitable for pasting into Excel, or as `json`
     */
    const copyLetters = (what: "text"|"table"|"json") => {
        if (what === "text") {
            let s = "";
            UPPERCASE.forEach(letter => {
                const num = letterNums.get(letter);
                if (num != null) {
                    for (let i=0; i<num; i++) {
                        s += letter;
                    }
                }
            });
            writeText(s);
        }
        else if (what === "table") {
            let s = "";
            UPPERCASE.forEach((letter, i) => {
                s += letter + "\t" + (letterNums.has(letter) ? letterNums.get(letter)! : "0");
                if (i < UPPERCASE.length-1) {
                    s += "\n";
                }
            });
            writeText(s);
        }
        else {
            const letters: Record<string, number> = {};
            UPPERCASE.forEach(letter => {
                if (!letterNums.has(letter)) {
                    letters[letter] = 0;
                }
                else {
                    letters[letter] = letterNums.get(letter)!;
                }
            });
            const j = JSON.stringify(letters, undefined, 4);
            writeText(j);
        }
    }

    /**
     * Tries to paste letters in different formats from the clipboard
     */
    const pasteLetters = () => {
        readText().then(val => {
            if (val != null) {
                // First check if it's in the "Copy as text" format
                const as_array = [...val].map(c => c.toUpperCase());
                if (as_array.every(c => UPPERCASE.includes(c))) {
                    const new_map = new Map<string, number>();
                    UPPERCASE.forEach(c => {
                        new_map.set(c, 0);
                    });
                    as_array.forEach(c => new_map.set(c, new_map.get(c)!+1));
                    setLetterNums(new_map);
                }
                else {
                    try {
                        const parsed = JSON.parse(val);
                        // Then check if it's a copied array
                        if (Array.isArray(parsed) && parsed.length === UPPERCASE.length && parsed.every(num => !isNaN(parseInt(num)) && parseInt(num) >= 0)) {
                            const new_map = new Map<string, number>();
                            UPPERCASE.forEach((c, i) => {
                                new_map.set(c, parseInt(parsed[i]));
                            });
                            setLetterNums(new_map);
                        }
                        // Then check if it's a copied JSON mapping (i.e. from "Copy as JSON")
                        else if (UPPERCASE.every(c => (parsed.hasOwnProperty(c) && !isNaN(parseInt(parsed[c])) && parseInt(parsed[c]) >= 0) || (parsed.hasOwnProperty(c.toLowerCase()) && !isNaN(parseInt(parsed[c.toLowerCase()])) && parseInt(parsed[c.toLowerCase()]) >= 0))) {
                            const new_map = new Map<string, number>();
                            UPPERCASE.forEach(c => {
                                let num = parsed.hasOwnProperty(c) && !isNaN(parseInt(parsed[c])) && parseInt(parsed[c]) >= 0 ? parseInt(parsed[c]) : parseInt(parsed[c.toLowerCase()]);
                                new_map.set(c, num)
                            });
                            setLetterNums(new_map);
                        }
                        // I don't think this else can ever be hit, but leaving it anyways
                        else {
                            const lines = val.split(/\r\n|[\n\v\f\r\x85\u2028\u2029]/); // Split at new lines; see https://www.unicode.org/reports/tr18/#Line_Boundaries
                            const new_map = new Map<string, number>();
                            for (let i=0; i<lines.length; i++) {
                                if (i >= UPPERCASE.length) {
                                    break;
                                }
                                const vals = lines[i].split(/\t/);
                                if (vals.length >= 2 && vals[0].toUpperCase() === UPPERCASE[i] && !isNaN(parseInt(vals[1])) && parseInt(vals[1]) >= 0) {
                                    new_map.set(UPPERCASE[i], parseInt(vals[1]));
                                }
                                else {
                                    break;
                                }
                            }
                            if (new_map.size === UPPERCASE.length) {
                                setLetterNums(new_map);
                            }
                        }
                    }
                    catch {
                        // If JSON parsing fails, try assuming it's form "Copy as table"
                        const lines = val.split(/\r\n|[\n\v\f\r\x85\u2028\u2029]/);
                        const new_map = new Map<string, number>();
                        for (let i=0; i<lines.length; i++) {
                            if (i >= UPPERCASE.length) {
                                break;
                            }
                            const vals = lines[i].split(/\t/);
                            if (vals.length >= 2 && vals[0].toUpperCase() === UPPERCASE[i] && !isNaN(parseInt(vals[1])) && parseInt(vals[1]) >= 0) {
                                new_map.set(UPPERCASE[i], parseInt(vals[1]));
                            }
                            else {
                                break;
                            }
                        }
                        if (new_map.size === UPPERCASE.length) {
                            setLetterNums(new_map);
                        }
                    }
                }
            }
        })
    }

    /**
     * Resets the hand of letters after confirmation
     */
    const resetLetters = () => {
        if (Array.from(letterNums.values()).some(val => val! > 0)) {
            const empty_letters = new Map();
            UPPERCASE.forEach(letter => {
                empty_letters.set(letter, 0);
            });
            confirmDialog({
                message: "Are you sure you want to reset the hand of letters?",
                header: "Reset?",
                icon: "pi pi-exclamation-triangle",
                accept: () => setLetterNums(empty_letters)
            });
        }
    }

    /**
     * Context menu items
     */
    const items: MenuItem[] = [
        { label: "Copy as text", icon: "pi pi-copy", command: () => copyLetters("text") },
        { label: "Copy as table", icon: "pi pi-file-excel", command: () => copyLetters("table")},
        { label: "Copy as JSON", icon: "pi pi-list", command: () => copyLetters("json") },
        { separator: true },
        { label: "Paste", icon: "pi pi-file-import", command: pasteLetters },
        { separator: true },
        { label: "Reset", icon: "pi pi-eraser", command: resetLetters }
    ];

    /**
     * Function for pasting into the letter input field
     */
    const pasteTypeIn = () => {
        readText().then(val => {
            if (val != null) {
                let s = "";
                let all_valid = true;
                for (const char of val) {
                    if (!UPPERCASE.includes(char.toUpperCase())) {
                        all_valid = false;
                        break;
                    }
                    s += char.toUpperCase();
                }
                if (all_valid) {
                    setTypedIn(s);
                }
            }
        });
    }

    /**
     * Function for pasting into the random number field
     */
    const pasteRandomNum = () => {
        readText().then(val => {
            if (val != null && [...val].every(char => DIGITS.includes(char))) {
                setRandomNum(val);
            }
        });
    }

    /**
     * Context menu items for the letter input field
     */
    const type_in_items: MenuItem[] = [
        { label: "Copy", icon: "pi pi-copy", command: () => writeText(typedIn) },
        { label: "Paste", icon: "pi pi-file-import", command: pasteTypeIn }
    ];

    /**
     * Context menu items for the random number input field
     */
    const random_num_items: MenuItem[] = [
        { label: "Copy", icon: "pi pi-copy", command: () => writeText(randomNum) },
        { label: "Paste", icon: "pi pi-file-import", command: pasteRandomNum }
    ];

    /**
     * Callback when a number is changed for a specified letter
     * @param c The letter of the number being changed
     * @param e The input change event
     */
    const changeLetterNum = (c: string, e: InputNumberValueChangeEvent) => {
        const new_map = new Map(letterNums);
        new_map.set(c, e.value);
        setLetterNums(new_map);
        const n = Number(e.value);
        if (!isNaN(n) && num_letters.get(c)! < n) {
            const new_map_invalid = new Map(lettersInvalid);
            new_map_invalid.set(c, true);
            setLettersInvalid(new_map_invalid);
        }
        else {
            const new_map_invalid = new Map(lettersInvalid);
            new_map_invalid.set(c, false);
            setLettersInvalid(new_map_invalid);
        }
    }

    /**
     * Counts the occurences of the `letter` in the string `s`
     * @param letter Letter to count in `s`
     * @param s String to count the occurences of `letter` in
     * @returns The number of times `letter` appears in `s`
     */
    const count_letter_in_string = (letter: string, s: string) => {
        let count = 0;
        for (let i=0; i < s.length; i++) {
            if (s[i] === letter) {
                count++;
            }
        }
        return count;
    }

    /**
     * Cancels the letter input by closing the popup and reseting all fields
     */
    const cancelInput = () => {
        setTypeInVisible(false);
        // So the dialog has time to finish its close animation before the fields are updated
        setTimeout(() => {
            setTypedIn("");
            setRandomNum("21");
            setRandomFrom("standard Bananagrams");
        }, 100);
    }

    /**
     * Callback when the "Use letters" button is clicked
     */
    const useLetters = () => {
        const new_map = new Map<string, number>();
        UPPERCASE.forEach(c => {
            new_map.set(c, count_letter_in_string(c, typedIn));
        });
        setLetterNums(new_map);
        cancelInput();
    }

    /**
     * Chooses random letters based on the user's input
     */
    const chooseRandomly = () => {
        const val = parseInt(randomNum);
        if (randomNum.trim() === "" || isNaN(val) || val <= 0) {
            props.toast.current?.show({severity: "warn", summary: "Invalid number", detail: "The number of letters to choose must be greater than 0"});
        }
        else if (val > 144 && randomFrom === "standard Bananagrams") {
            props.toast.current?.show({severity: "warn", summary: "Too many letters", detail: "No more than 144 tiles can be chosen from standard Bananagrams"});
        }
        else if (val > 288 && randomFrom === "double Bananagrams") {
            props.toast.current?.show({severity: "warn", summary: "Too many letters", detail: "No more than 288 tiles can be chosen from double Bananagrams"});
        }
        else {
            invoke("get_random_letters", {what: randomFrom, howMany: val}).then(res => {
                const result = res as Record<string, number>;
                const new_map = new Map<string, number>();
                UPPERCASE.forEach(c => {
                    new_map.set(c, result[c] ?? 0);
                });
                setLetterNums(new_map);
                cancelInput();
            })
            .catch(error => props.toast.current?.show({severity: "error", summary: "Error generating letters", detail: "An error occurred generating random letters: " + error}));
        }
    }

    /**
     * Attempts to display words makeable with the hand of letters
     */
    const viewPlayableWords = () => {
        let s = 0;
        for (const value of letterNums.values()) {
            s += value ?? 0;
        }
        if (s < 2) {
            props.toast.current?.show({"severity": "warn", "summary": "Not enough letters", "detail": "More than one letter must be present."})
        }
        else {
            setPlayableWordsLoading(true);
            const letters = new Map<string, number>();
            UPPERCASE.forEach(c => {
                letters.set(c, letterNums.get(c) ?? 0);
            });
            invoke("get_playable_words", { availableLetters: letters }).then(res => {
                const result = res as {short: string[], long: string[]};
                props.setPlayableWords(result);
                props.setPlayableWordsVisible(true);
            })
            .catch(error => props.toast.current?.show({"severity": "warn", "summary": "An error occurred", "detail": "An error occurred getting the playable words: " + error}))
            .finally(() => setPlayableWordsLoading(false));
        }
    }

    /**
     * Resets either the hand or board after confirmation
     * @param which Whether to reset the hand ("Reset hand") or the board ("Reset board")
     */
    const doReset = (which: "Reset hand"|"Reset board") => {
        if (which === "Reset hand") {
            resetLetters();
        }
        else {
            confirmDialog({
                message: "Are you sure you want to reset the board?",
                header: "Reset?",
                icon: "pi pi-exclamation-triangle",
                accept: props.clearResults
            });
        }
    }

    /**
     * Callback to start solving the puzzle
     */
    const solve = () => {
        let s = 0;
        for (const value of letterNums.values()) {
            s += value ?? 0;
        }
        if (s < 2) {
            props.toast.current?.show({"severity": "warn", "summary": "Not enough letters", "detail": "More than one letter must be present."})
        }
        else {
            const letters = new Map<string, number>();
            UPPERCASE.forEach(c => {
                letters.set(c, letterNums.get(c) ?? 0);
            });
            props.startRunning(letters);
        }
    }

    /**
     * Performs an undo
     */
    const undo = () => {
        invoke("undo").then((res: any) => {
            const new_map = new Map<string, number>();
            UPPERCASE.forEach(c => {
                new_map.set(c, res.letters[c] ?? 0);
            });
            setLetterNums(new_map);
            props.setUndoPossible(res.undo_possible);
            props.setRedoPossible(res.redo_possible);
            props.setResults(res.solution.length > 0 ? { elapsed: 0, board: res.solution } : null);
        })
        .catch(err => props.toast.current?.show({severity: "error", summary: "Could not undo", detail: err.toString()}));
    }

    /**
     * Performs a redo
     */
    const redo = () => {
        invoke("redo").then((res: any) => {
            console.log(res);
            const new_map = new Map<string, number>();
            UPPERCASE.forEach(c => {
                new_map.set(c, res.letters[c] ?? 0);
            });
            setLetterNums(new_map);
            props.setUndoPossible(res.undo_possible);
            props.setRedoPossible(res.redo_possible);
            props.setResults(res.solution.length > 0 ? { elapsed: 0, board: res.solution } : null);
        })
        .catch(err => props.toast.current?.show({severity: "error", summary: "Could not undo", detail: err.toString()}));
    }
    
    return (
        <>
        <ContextMenu model={items} ref={cm}/>
        <ContextMenu model={type_in_items} ref={type_in_cm}/>
        <ContextMenu model={random_num_items} ref={random_num_cm}/>
        {[...individual_cms]}
        <Dialog header="Input letters" visible={typeInVisible} onHide={() => setTypeInVisible(false)}>
            <TabView>
                <TabPanel header="Type in letters">
                    <form onSubmit={e => {e.preventDefault(); useLetters()}} autoComplete="off">
                        <InputText value={typedIn} onChange={e => setTypedIn(e.target.value.toUpperCase())} keyfilter="alpha" id="typeIn" onContextMenu={e => type_in_cm.current?.show(e)}/>
                        <br/>
                        <Button type="submit" label="Use letters" icon="pi pi-arrow-right" iconPos="right" style={{marginTop: "5px", marginRight: "5px"}}/>
                        <Button type="reset" label="Cancel" icon="pi pi-times" iconPos="right" severity="secondary" onClick={() => cancelInput()}/>
                    </form>
                </TabPanel>
                <TabPanel header="Choose randomly">
                    <form onSubmit={e=> {e.preventDefault(); chooseRandomly()}} autoComplete="off">
                        <span>Choose </span>
                        <InputText value={randomNum} onChange={e => setRandomNum(e.target.value)} keyfilter="int" size={3} onContextMenu={e => random_num_cm.current?.show(e)}/>
                        <span> random letters from </span>
                        <Dropdown value={randomFrom} onChange={e => setRandomFrom(e.value)} options={["standard Bananagrams", "double Bananagrams", "infinite set"]}/>
                        <br/>
                        <Button type="submit" label="Choose letters" icon="pi pi-arrow-right" iconPos="right" style={{marginTop: "5px", marginRight: "5px"}}/>
                        <Button type="reset" label="Cancel" icon="pi pi-times" iconPos="right" severity="secondary" onClick={() => cancelInput()}/>
                    </form>
                </TabPanel>
            </TabView>            
        </Dialog>
        <div>
            {UPPERCASE.map((c, i) => {
                return (
                    <span className="letter-input-span" key={"span-"+c}><label htmlFor={"char-"+c} className="letter-input-label">{c}:</label>
                        <InputNumber inputId={"char-"+c} value={letterNums.get(c)} onValueChange={e => changeLetterNum(c, e)} min={0} size={1} showButtons inputStyle={{padding: "5px", width: "3rem"}} incrementButtonClassName="input-button-type" decrementButtonClassName="input-button-type" className={lettersInvalid.get(c) ? "p-invalid" : undefined} style={{marginTop: "5px", paddingLeft: "5px"}} onContextMenu={e => individual_cm_refs[i].current?.show(e)}/>
                    </span>
                )
            })}
            <span style={{whiteSpace: "nowrap", marginLeft: "15px", marginRight: "5px"}}><b>Total letters:</b> {Array.from(letterNums.values()).reduce((previousValue, val) => previousValue!+val!, 0)}</span>
        </div>
        <br/>
        <div className="button-div">
            <Button type="button" label="Input letters" icon="pi pi-book" iconPos="right" style={{padding: "8px", marginRight: "2%"}} onClick={() => setTypeInVisible(true)}/>
            <Button type="button" label="View playable words" icon="pi pi-eye" iconPos="right" style={{padding: "8px",}} onClick={viewPlayableWords} loading={playableWordsLoading}/>
        </div>
        <div className="button-div">
            <Dropdown placeholder="Reset" options={["Reset hand", "Reset board"]} style={{marginRight: "2%"}} onChange={e => doReset(e.value)} className="reset-dropdown" panelClassName="reset-dropdown" pt={{input: {style: {color: "white"}}, item: {className: "reset-dropdown-item"}, trigger: {style: {color: "white"}}}}/>
            <Button type="button" label="Solve" icon="pi pi-arrow-right" iconPos="right" severity="success" onClick={solve} loading={props.running}/>
        </div>
        <div className="button-div" style={{marginTop: "15px"}}>
            <Button label="Undo" icon="pi pi-undo" iconPos="right" onClick={undo} style={{marginRight: "2%"}} disabled={!props.undoPossible}/>
            <Button label="Redo" icon="pi pi-refresh" iconPos="right" onClick={redo} disabled={!props.redoPossible}/>
        </div>
        </>
    )
}