import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { CopyProvider } from "../../copy/CopyProvider";
import { Beast } from "./Beast";

describe("Beast", () => {
  it("shows the mood in English", () => {
    render(
      <CopyProvider locale="en">
        <Beast mood="kaiju" />
      </CopyProvider>,
    );
    expect(screen.getByText("KAIJU MODE. Evacuate the city.")).toBeInTheDocument();
    expect(document.querySelector("[data-mood=kaiju]")).not.toBeNull();
  });

  it("shows the mood in Japanese", () => {
    render(
      <CopyProvider locale="ja">
        <Beast mood="asleep" />
      </CopyProvider>,
    );
    expect(screen.getByText("おやすみ中。起こさないで。")).toBeInTheDocument();
    expect(document.documentElement.lang).toBe("ja");
  });
});
