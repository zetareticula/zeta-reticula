import React, { useEffect, useState } from "react";
import { fetchInferenceStats } from "../api";
import { Chart as ChartJS, ArcElement, Tooltip, Legend, LineElement, PointElement, LinearScale, TimeScale } from "chart.js";
import { Doughnut, Line } from "react-chartjs-2";
import "chartjs-adapter-date-fns";

ChartJS.register(ArcElement, Tooltip, Legend, LineElement, PointElement, LinearScale, TimeScale);

const StatsPanel = () => {
  const [stats, setStats] = useState({
    latency: 0.4,
    memory_savings: 60,
    throughput: 2500,
    anns_recall: 0.95,
  });

  useEffect(() => {
    const loadStats = async () => {
      const inferenceStats = await fetchInferenceStats();
      setStats(inferenceStats);
    };
    loadStats();
    const interval = setInterval(loadStats, 5000); // Update every 5 seconds
    return () => clearInterval(interval);
  }, []);

  const latencyData = {
    labels: ["Current Latency"],
    datasets: [
      {
        label: "Latency (ms/sample)",
        data: [stats.latency, 1 - stats.latency], // Mock for visualization
        backgroundColor: ["#6b5be3", "#2e335a"],
        borderColor: ["#a29bfe", "#2e335a"],
        borderWidth: 1,
      },
    ],
  };

  const memoryData = {
    labels: ["Memory Savings"],
    datasets: [
      {
        label: "Memory Savings (%)",
        data: [stats.memory_savings, 100 - stats.memory_savings],
        backgroundColor: ["#6b5be3", "#2e335a"],
        borderColor: ["#a29bfe", "#2e335a"],
        borderWidth: 1,
      },
    ],
  };

  const throughputData = {
    labels: new Array(10).fill().map((_, i) => new Date(Date.now() - (9 - i) * 1000)),
    datasets: [
      {
        label: "Throughput (queries/s)",
        data: new Array(10).fill(stats.throughput), // Mock historical data
        borderColor: "#a29bfe",
        backgroundColor: "#6b5be3",
        fill: false,
        tension: 0.3,
      },
    ],
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
      {/* Latency Panel */}
      <div className="bg-cosmic-light p-6 rounded-lg shadow-glow">
        <h3 className="text-lg font-bold text-cosmic-glow mb-4">Latency</h3>
        <Doughnut data={latencyData} options={{ plugins: { legend: { labels: { color: "#a29bfe" } } } }} />
        <p className="text-center mt-4 text-cosmic-glow">{stats.latency} ms/sample</p>
      </div>

      {/* Memory Savings Panel */}
      <div className="bg-cosmic-light p-6 rounded-lg shadow-glow">
        <h3 className="text-lg font-bold text-cosmic-glow mb-4">Memory Savings</h3>
        <Doughnut data={memoryData} options={{ plugins: { legend: { labels: { color: "#a29bfe" } } } }} />
        <p className="text-center mt-4 text-cosmic-glow">{stats.memory_savings}%</p>
      </div>

      {/* Throughput Panel */}
      <div className="bg-cosmic-light p-6 rounded-lg shadow-glow">
        <h3 className="text-lg font-bold text-cosmic-glow mb-4">Throughput</h3>
        <Line data={throughputData} options={{ plugins: { legend: { labels: { color: "#a29bfe" } } }, scales: { x: { type: "time", time: { unit: "second" }, ticks: { color: "#a29bfe" } }, y: { ticks: { color: "#a29bfe" } } } }} />
        <p className="text-center mt-4 text-cosmic-glow">{stats.throughput} queries/s</p>
      </div>

      {/* Additional Panel for ANNS Recall */}
      <div className="bg-cosmic-light p-6 rounded-lg shadow-glow">
        <h3 className="text-lg font-bold text-cosmic-glow mb-4">ANNS Recall</h3>
        <p className="text-center text-3xl text-cosmic-glow">{(stats.anns_recall * 100).toFixed(2)}%</p>
      </div>

      {/* Placeholder for Future Panels */}
      <div className="bg-cosmic-light p-6 rounded-lg shadow-glow">
        <h3 className="text-lg font-bold text-cosmic-glow mb-4">Sparsity Ratio</h3>
        <p className="text-center text-3xl text-cosmic-glow">Coming Soon...</p>
      </div>

      <div className="bg-cosmic-light p-6 rounded-lg shadow-glow">
        <h3 className="text-lg font-bold text-cosmic-glow mb-4">Federated Clients</h3>
        <p className="text-center text-3xl text-cosmic-glow">Coming Soon...</p>
      </div>
    </div>
  );
};

export default StatsPanel;