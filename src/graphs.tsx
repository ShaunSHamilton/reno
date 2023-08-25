import { useEffect, useState } from "react";
import { Line } from "react-chartjs-2";
import {
  CategoryScale,
  Chart,
  LineElement,
  LinearScale,
  PointElement,
} from "chart.js";

Chart.register(CategoryScale, LinearScale, PointElement, LineElement);

type Stat = {
  timestamp: number;
  volt: number;
};

export function Graphs() {
  const [stats, setStats] = useState<Stat[]>([]);

  useEffect(() => {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    async function getStats() {
      const stats = [
        { timestamp: 0, volt: 13.1 },
        { timestamp: 1, volt: 13.2 },
      ];
      setStats(stats);
    }
    getStats();
  }, []);

  return (
    <section>
      <Line
        datasetIdKey="volt"
        data={{
          labels: stats.map(({ timestamp }) => timestamp),
          datasets: [
            {
              label: "Voltage",
              data: stats.map(({ volt, timestamp }) => {
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
