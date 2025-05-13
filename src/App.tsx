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
//import { Slider } from "primereact/slider";
import { Dropdown } from "primereact/dropdown";
import DigitalNumber from "react-digital-number/dist/index.js";
import Slider from "@mui/material/Slider";

function App() {
  const [bpm, setBpm] = useState(120);
  const [volume, setVolume] = useState(50);
  const [timeSig, setTimeSig] = useState<TimeSignature>({ top: 4, bottom: 4 });
  const [started, isStarted] = useState(false);
  const [beatIndex, setIndex] = useState(0);

  const allTicks = Array.from({ length: 21 }, (_, i) => 100 - i * 5);

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
    <main className="container w-full h-full">
      <div className="flex flex-row w-full justify-center">
        <div className="volume-component flex flex-row items-center">
          <div className="flex flex-column justify-content-between ">
            {[100, 80, 60, 40, 20, 0].map((val) => (
              <span className="flex justify-content-center bg-primary text-sm">
                {val}
              </span>
            ))}
          </div>
          <div className="tick-col flex flex-column justify-between align-items-end">
            {allTicks.map((_) => (
              <hr className="separator m-2 flex justify-content-start" />
            ))}
          </div>
          <Slider
            orientation="vertical"
            min={0}
            max={100}
            value={volume}
            onChange={(_, val) => setVolume(val as number)}
            sx={{ height: "100%" }}
          />
        </div>
        <div className="bpm-container">
          <div className="digital-component">
            <DigitalNumber
              nums={bpm.toString().padStart(3, "0")}
              color="#ffffff"
              unActiveColor="#22221e"
              backgroundColor="transparent"
              width={150}
              height={80}
            />
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
              <Scale tickWidth={2} tickHeight={2} radius={70} color="#808080" />
              <circle r="50" cx="75" cy="75" fill="#808080" />
              <Pointer
                width={3}
                height={35}
                radius={23}
                type="rect"
                color="#808080"
              />

              <text
                x="75"
                y="80"
                textAnchor="middle"
                fill="#000"
                fontSize="20"
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
      <div className="time-sig-component flex flex-row w-full pt-3">
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
      <div>
        <button
          onClick={() => {
            toggleMetronome();
          }}
        >
          {started ? "Stop" : "Start"}
        </button>
      </div>
    </main>
  );
}

export default App;
