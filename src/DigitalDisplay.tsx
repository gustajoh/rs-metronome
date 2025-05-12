// DigitalDisplay.tsx
export const DigitalDisplay = ({
  value,
  color = "#0f0",
  fontSize = "5rem",
  background = "#000",
  widthChars = 3,
}: {
  value: number | string;
  color?: string;
  fontSize?: string;
  background?: string;
  widthChars: number;
}) => {
  const displayValue = value.toString().padStart(widthChars, " "); // pad with spaces

  return (
    <div
      style={{
        fontFamily: "monospace",
        fontSize,
        color,
        background,
        padding: "1rem 2rem",
        borderRadius: "8px",
        textAlign: "center",
        boxShadow: `0 0 15px ${color}`,
        userSelect: "none",
        minWidth: "12rem",
      }}
    >
      {displayValue}
    </div>
  );
};
