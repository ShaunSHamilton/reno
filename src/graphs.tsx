import { useContext, useEffect, useState } from "react";
import { Line } from "react-chartjs-2";
import {
  CategoryScale,
  Chart,
  LineElement,
  LinearScale,
  PointElement,
} from "chart.js";
import { DataContext } from "./state";

Chart.register(CategoryScale, LinearScale, PointElement, LineElement);

export function Graphs() {
  const data = useContext(DataContext);

  const levels = data.filter(({ data }) => data.Levels);

  return (
    <section>
      <Line
        datasetIdKey="volt"
        data={{
          labels: levels.map(({ timestamp }) => timestamp),
          datasets: [
            {
              label: "Voltage",
              data: levels.map(({ data, timestamp }) => {
                const volt = data.Levels?.volt ?? 0;
                return {
                  y: volt,
                  x: timestamp,
                };
              }),
              borderColor: "red",
            },
          ],
        }}
      />
    </section>
  );
}
