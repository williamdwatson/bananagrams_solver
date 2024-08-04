import { invoke } from "@tauri-apps/api";
import { Button } from "primereact/button"
import { Dialog } from "primereact/dialog";
import { Dropdown } from "primereact/dropdown";
import { InputNumber } from "primereact/inputnumber";
import { OverlayPanel } from "primereact/overlaypanel";
import { Toast } from "primereact/toast"
import { Tooltip } from "primereact/tooltip";
import { RefObject, useEffect, useRef, useState } from "react"

interface SettingsProps {
    /**
     * Toast popup reference
     */
    toast: RefObject<Toast>
}

export default function Settings(props: SettingsProps) {
    const [showSettings, setShowSettings] = useState(false);
    const [filterLettersOnBoard, setFilterLettersOnBoard] = useState<number|null>(2);
    const [whichDictionary, setWhichDictionary] = useState<"Short"|"Full">("Short");
    const [maximumWordsToCheck, setMaximumWordsToCheck] = useState<number|null>(50000);
    const filterLettersInfo = useRef<OverlayPanel>(null);
    const maxWordsInfo = useRef<OverlayPanel>(null);
    const whichDictionaryInfo = useRef<OverlayPanel>(null);

    // Get the available settings whenever the popup is shown
    useEffect(() => {
        if (showSettings) {
            invoke("get_settings").then(res => {
                console.log(res);
            })
        }
    }, [showSettings]);

    /**
     * Updates the settings
     */
    const setSettings = () => {
        if (filterLettersOnBoard == null) {
            props.toast.current?.show({severity: "warn", summary: "Missing usable letters on board", detail: "The number of usable letters on the board must be provided"});
        }
        else if (filterLettersOnBoard < 0) {
            props.toast.current?.show({severity: "warn", summary: "Invalid usable letters on board", detail: "The number of usable letters on the board must be a non-negative integer."})
        }
        else if (filterLettersOnBoard >= 2**32) {
            props.toast.current?.show({severity: "warn", summary: "Invalid usable letters on board", detail: "The number of usable letters on the board must be less than 2³²"});
        }
        else if (maximumWordsToCheck == null) {
            props.toast.current?.show({severity: "warn", summary: "Missing maximum iterations", detail: "The maximum iterations must be provided"});
        }
        else if (maximumWordsToCheck < 0) {
            props.toast.current?.show({severity: "warn", summary: "Invalid maximum iterations", detail: "The maximum iterations must be a non-negative integer"});
        }
        else if (maximumWordsToCheck >= 2**32) {
            props.toast.current?.show({severity: "warn", summary: "Invalid maximum iterations", detail: "The maximum iterations must be less than 2³²"});
        }
        else {
            invoke("set_settings", { filterLettersOnBoard, maximumWordsToCheck, useLongDictionary: whichDictionary === "Full"})
            .then(() => setShowSettings(false))
            .catch(err => props.toast.current?.show({severity: "error", summary: "Error updating settings", detail: `An error occurred: ${err}`}));
        }
    }

    return (
        <>
        <Dialog header="Settings" visible={showSettings} onHide={() => setShowSettings(false)}>
            <div className="settings-div">
                <label htmlFor="filter_letters_on_board">Usable letters on board:</label> <InputNumber value={filterLettersOnBoard} onChange={e => setFilterLettersOnBoard(e.value)} min={0} inputId="filter_letters_on_board"/>
                <OverlayPanel ref={filterLettersInfo} style={{maxWidth: "33vw"}}>
                    <p>The maximum number of letters on the board that can be used in conjuction with letters in the hand when filtering playable words</p>
                    <p><strong>Lower values:</strong> <em>Usually</em> faster solutions</p>
                    <p><strong>Higher values:</strong> <em>Usually</em> slower solutions, but more likely to find a solution if one exists. For an exhaustive search, use a value greater than the total number of letters.</p>
                </OverlayPanel>
                <i className="pi pi-info-circle info-overlay" onClick={e => filterLettersInfo.current?.toggle(e)}></i>
            </div>
            <div className="settings-div">
                <label htmlFor="max_words_to_check">Maximum iterations:</label> <InputNumber value={maximumWordsToCheck} onChange={e => setMaximumWordsToCheck(e.value)} min={0} inputId="max_words_to_check"/>
                <OverlayPanel ref={maxWordsInfo} style={{maxWidth: "33vw"}}>
                    <p>The maximum number of iterations before the solver stops and returns no solution (i.e. a "dump")</p>
                    <p><strong>Lower values:</strong> Faster "dump" solutions</p>
                    <p><strong>Higher values:</strong> Slower "dump" solutions, but more likely to find a solution if one exists. For an exhaustive search, use a very large value.</p>
                </OverlayPanel>
                <i className="pi pi-info-circle info-overlay" onClick={e => maxWordsInfo.current?.toggle(e)}></i>
            </div>
            <div className="settings-div">
                <label htmlFor="use_dictionary">Dictionary:</label> <Dropdown value={whichDictionary} onChange={e => setWhichDictionary(e.value)} options={["Short", "Full"]} inputId="use_dictionary"/>
                <OverlayPanel ref={whichDictionaryInfo} style={{maxWidth: "33vw"}}>
                    <p>Which dictionary to use</p>
                    <p><strong>Short:</strong> Contains 30,522 words</p>
                    <p><strong>Full:</strong> Contains 178,691 words, including some that some players might consider questionable</p>
                </OverlayPanel>
                <i className="pi pi-info-circle info-overlay" onClick={e => whichDictionaryInfo.current?.toggle(e)}></i>
            </div>
            <div>
                <Button label="Use settings" icon="pi pi-arrow-right" iconPos="right" onClick={setSettings}/>
                <Button label="Cancel" icon="pi pi-times" iconPos="right" severity="secondary" onClick={() => setShowSettings(false)} style={{marginLeft: "5px"}}/>
            </div>
        </Dialog>
        <div style={{marginTop: "5vh", width: "100%", display: "flex", justifyContent: "center"}}>
            <Button label="Settings" icon="pi pi-cog" iconPos="right" onClick={() => setShowSettings(true)}/>
        </div>
        </>
    )
}