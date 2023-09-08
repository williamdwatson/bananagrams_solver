import { useRef, useState } from "react";
import { Button } from "primereact/button";
import { Checkbox } from "primereact/checkbox";
import { ContextMenu } from "primereact/contextmenu";
import { Dropdown } from "primereact/dropdown";
import { InputSwitch } from "primereact/inputswitch";
import { InputText } from "primereact/inputtext";
import { MenuItem } from "primereact/menuitem";
import { VirtualScroller, VirtualScrollerTemplateOptions } from "primereact/virtualscroller";
import { useDebounce } from "primereact/hooks";
import { readText, writeText } from "@tauri-apps/api/clipboard";
import PlayableWordsStats from "./playable_words_stats";

interface PlayableWordsListProps {
    /**
     * List of playable words
     */
    playableWords: string[]|undefined,
    /**
     * Which set of playable words (used for IDs)
     */
    which: string
}

type filter_types = "Starts with"|"Does not start with"|"Contains"|"Does not contain"|"Ends with"|"Does not end with";

/**
 * Shows a list of playable words
 * 
 * @component
 */
export default function PlayableWordsList(props: PlayableWordsListProps) {
    const cm = useRef<ContextMenu|null>(null);
    const filter_cm = useRef<ContextMenu|null>(null);
    const [sort, setSort] = useState<boolean>(false);
    const [checked, setChecked] = useState<boolean|undefined>(false);
    const [filter, debouncedFilter, setFilter] = useDebounce("", 400);
    const [filterType, setFilterType] = useState<filter_types>("Starts with");
    const [showStats, setShowStats] = useState(false);

    /**
     * Formats a row displaying a playable word
     * @param item Word to display
     * @param options Options associated with the virtual scroller
     * @returns The properly formatted word row
     */
    const playableWordTemplate = (item: string, options: VirtualScrollerTemplateOptions) => {
        return (
            <div style={{ backgroundColor: options.odd ? "rgb(246, 249, 252)" : "white", display: "flex", alignItems: "center", justifyContent: "center", height: options.props.itemSize + 'px' }}>
                {item}
            </div>
        );
    }

    /**
     * Filtering function for playable words
     * @param word Word to check if passes the filters
     * @returns Whether `word` passes the user entered filter
     */
    const checkFilter = (word: string) => {
        if (debouncedFilter) {
            if (filterType === "Starts with") {
                return word.startsWith(debouncedFilter);
            }
            else if (filterType === "Does not start with") {
                return !word.startsWith(debouncedFilter);
            }
            else if (filterType === "Contains") {
                return word.includes(debouncedFilter);
            }
            else if (filterType === "Does not contain") {
                return !word.includes(debouncedFilter);
            }
            else if (filterType === "Ends with") {
                return word.endsWith(debouncedFilter);
            }
            else {
                return !word.endsWith(debouncedFilter);
            }
        }
        else {
            return true;
        }
    }

    /**
     * Sort function for playable words
     * @param a First word to compare
     * @param b Second word to comparse
     * @returns Comparison of `a` and `b` for sorting based on the user selections
     */
    const sortPlayable = (a: string, b: string) => {
        // If the reverse checkbox is set, then swap the variables
        if (checked) {
            [a, b] = [b, a];
        }
        // Sort by length or alphabetically, then the other (see https://stackoverflow.com/a/44554205)
        if (sort) {
            return a.length - b.length || a.localeCompare(b);
        }
        else {
            return a.localeCompare(b) || a.length - b.length;
        }
    }

    /**
     * Copies the playable words as text
     */
    const copyAsTable = () => {
        const words = props.playableWords?.filter(checkFilter).sort(sortPlayable);
        if (words && words.length > 0) {
            let s = "";
            words.forEach((word, i) => {
                s += word;
                if (i !== words!.length-1) {
                    s += "\n";
                }
            });
            writeText(s);
        }
    }

    /**
     * Copies the playable words as JSON
     */
    const copyAsJson = () => {
        const words = props.playableWords?.filter(checkFilter).sort(sortPlayable);
        if (words && words.length > 0) {
            writeText(JSON.stringify(words, null, 4));
        }
    }

    /**
     * Context menu items
     */
    const items: MenuItem[] = [
        { label: "Copy as table", icon: "pi pi-file-excel", command: copyAsTable },
        { label: "Copy as JSON", icon: "pi pi-list", command: copyAsJson }
    ];

    /**
     * Filter input context menu items
     */
    const filter_items: MenuItem[] = [
        { label: "Copy", icon: "pi pi-copy", command: () => writeText(filter) },
        { label: "Paste", icon: "pi pi-file-import", command: async () => setFilter((await readText()) ?? "") }
    ];

    const sorted_and_filtered = props.playableWords?.filter(checkFilter).sort(sortPlayable);

    return (
        <>
        <PlayableWordsStats playableWords={props.playableWords} visible={showStats} setVisible={setShowStats} type={props.which === "full" ? "all" : props.which}/>
        <div onContextMenu={e => cm.current?.show(e)}>
            <ContextMenu model={items} ref={cm}/>
            <ContextMenu model={filter_items} ref={filter_cm}/>
            {sorted_and_filtered != null ? 
                <div>
                    <div className="playable-word-sort-filter">
                        <span style={{fontWeight: "bold"}}>Sort:</span>
                        <span style={{marginLeft: "5px"}}>Alphabetically</span>
                        <InputSwitch checked={sort} onChange={e => setSort(e.value ?? false)} style={{margin: "0 5px"}}/>
                        <span style={{marginRight: "10px"}}>By length</span>
                        <Checkbox inputId={props.which+"_checkbox"} onChange={e => setChecked(e.checked)} checked={checked ?? false}/> <label htmlFor={props.which+"_checkbox"} style={{marginLeft: "5px"}}>Reverse</label>
                    </div>
                    <div className="playable-word-sort-filter">
                        <span style={{fontWeight: "bold"}}>Filter:</span>
                        <InputText value={filter} onChange={e => setFilter(e.target.value.toUpperCase())} keyfilter="alpha" style={{margin: "0 5px"}} onContextMenu={e => filter_cm.current?.show(e)}/>
                        <Dropdown value={filterType} onChange={e => setFilterType(e.value)} options={["Starts with", "Does not start with", "Contains", "Does not contain", "Ends with", "Does not end with"]}/>
                    </div>
                    <VirtualScroller items={sorted_and_filtered} itemSize={40} itemTemplate={playableWordTemplate} style={{ width: "30vw", height: "50vh" }} />
                    <Button type="button" label="See stats" icon="pi pi-calculator" iconPos="right" onClick={() => setShowStats(true)} style={{marginTop: "5px"}}/>
            </div>
            : null}
        </div>
        </>
    )
}