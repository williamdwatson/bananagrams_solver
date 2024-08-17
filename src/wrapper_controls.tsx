import { Button } from "primereact/button";
import { useControls } from "react-zoom-pan-pinch";

export default function WrapperControls() {
    const controls = useControls();
    return (
        // Glass effect from https://css.glass/
        <div style={{position: "fixed", top: "6%", right: "20px", display: "grid", rowGap: "5px", zIndex: 2,
                    background: "rgba(255, 255, 255, 0.2)", borderRadius: "16px",
                    backdropFilter: "blur(5px)", padding: "10px", border: "1px solid rgba(200, 200, 200, 0.45)"
        }}>
            <Button icon="pi pi-search-plus" rounded outlined severity="info" onClick={() => controls.zoomIn()}/>
            <Button icon="pi pi-search-minus" rounded outlined severity="info" onClick={() => controls.zoomOut()}/>
            <Button icon="pi pi-home" rounded outlined severity="info" onClick={() => controls.resetTransform()}/>
        </div>
    )
}