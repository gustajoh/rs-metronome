import "primereact/resources/themes/lara-light-indigo/theme.css";
import "primereact/resources/primereact.min.css";
import "primeicons/primeicons.css";
import "primeflex/primeflex.css";
import "./App.css";

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { TimeSignature } from "./types/timesignature";
import { listen } from "@tauri-apps/api/event";
import { Knob, Arc, Pointer, Value, Scale } from "rc-knob";
import { Slider } from "primereact/slider";
import { Tooltip } from "primereact/tooltip";
import { Dropdown } from "primereact/dropdown";
import { DigitalDisplay } from "./DigitalDisplay";

function App() {
  const [bpm, setBpm] = useState(120);
  const [volume, setVolume] = useState(50);
  const [timeSig, setTimeSig] = useState<TimeSignature>({ top: 4, bottom: 4 });
  const [started, isStarted] = useState(false);
  const [beatIndex, setIndex] = useState(0);

  const TIME_SIGNATURES: TimeSignature[] = [
    { top: 2, bottom: 4 },
    { top: 3, bottom: 4 },
    { top: 4, bottom: 4 },
    { top: 5, bottom: 4 },
    { top: 5, bottom: 8 },
    { top: 6, bottom: 8 },
    { top: 7, bottom: 8 },
    { top: 8, bottom: 8 },
  ];

  const timeSigOptions = TIME_SIGNATURES.map((sig) => ({
    label: `${sig.top}/${sig.bottom}`,
    value: sig,
  }));

  const toggleMetronome = async () => {
    if (started) {
      await invoke("stop_metronome");
    } else {
      try {
        await invoke("start_metronome", {
          bpm,
          timeSignature: timeSig,
          volume: volume / 100,
        });
      } catch (e) {
        console.error("Failed to start metronome:", e);
      }
    }
    isStarted(!started);
  };

  useEffect(() => {
    const unlisten = listen<number>("tick", (event) => {
      console.log("Tick event received, beat index:", event.payload);
      setIndex(event.payload);
    });

    return () => {
      unlisten.then((off) => off());
    };
  }, []);

  useEffect(() => {
    if (started) {
      const update = async () => {
        await invoke("update_metronome", {
          bpm,
          timeSignature: timeSig,
          volume: volume / 100,
        });
      };

      update().catch();
    }
  }, [volume, bpm, timeSig]);

  return (
    <main className="container w-full">
      <div className="flex flex-row w-full justify-center gap-4">
        <div className="volume-component flex flex-row align-content-start w-4">
          <Tooltip
            className="vol-tooltip flex sm"
            target=".volume-slider>.p-slider-handle"
            content={`${volume}%`}
            position="top"
          />
          <Slider
            className="volume-slider"
            value={volume}
            onChange={(e: any) => setVolume(e.value)}
          />{" "}
          <i className="pi pi-volume-up text-xl text-cyan-100" />
        </div>
        <div className="bpm-container">
          <div className="display-component">
            <DigitalDisplay value={bpm} fontSize="4rem" />
          </div>
          <div className="knob-component">
            <Knob
              size={150}
              angleOffset={250}
              angleRange={220}
              steps={200}
              min={40}
              max={240}
              snap={true}
              value={bpm}
              onChange={(value: any) => setBpm(parseInt(value))}
            >
              <Scale tickWidth={2} tickHeight={2} radius={45} color="#ffff" />
              <circle r="33" cx="75" cy="75" fill="#ffff" />
              <Pointer
                width={2}
                height={35}
                radius={4}
                type="rect"
                color="#ffff"
              />

              <text
                x="75"
                y="80"
                textAnchor="middle"
                fill="#000"
                fontSize="15"
                fontFamily="monospace"
                fontWeight="bold"
                style={{ userSelect: "none" }}
              >
                BPM
              </text>
            </Knob>
          </div>
        </div>
      </div>
      <div className="time-sig-component flex flex-row w-full">
        <div className="dropdown-component w-4 pr-8">
          <Dropdown
            value={timeSig}
            options={timeSigOptions}
            onChange={(e) => {
              setTimeSig(e.value);
            }}
          />
        </div>
        <div className="beat-visualizer-component">
          {[...Array(timeSig.top)].map((_, i) => (
            <svg key={i} width="50" height="50">
              <circle
                cx="15"
                cy="15"
                r="12"
                fill={
                  beatIndex === i
                    ? i === 0
                      ? "#00ff5e"
                      : "#ffffff"
                    : "#999999"
                }
                stroke="#000"
                strokeWidth="1"
                style={{ transition: "fill 0.1s ease-in-out" }}
              />
            </svg>
          ))}
        </div>
      </div>
      <br />
      <button
        onClick={() => {
          toggleMetronome();
        }}
      >
        {started ? "Stop" : "Start"}
      </button>
      <style></style>
    </main>
  );
}

export default App;
