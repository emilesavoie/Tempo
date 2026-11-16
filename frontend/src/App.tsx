
import { useEffect, useState } from "react";

import {
  Activity,
  Bike,
  CalendarDays,
  ChartNoAxesCombined,
  Clock3,
  Footprints,
  LayoutDashboard,
  Settings,
  Waves,
  Bell,
  ChevronDown,
  Route,
  UserRound,
  HeartPulse,
  Gauge,
  RefreshCw,
  Dumbbell,
} from "lucide-react";

import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";

// -----------------------------------------------------------------------------
// API types
// -----------------------------------------------------------------------------

type SportStats = {
  sessions: number;
  hours: number;
  load: number;
  distance_km: number;
};

type WeekSummary = {
  week_start: string;
  partial: boolean;
  by_sport: {
    swim?: SportStats;
    bike?: SportStats;
    run?: SportStats;
    other?: SportStats;
  };
  total_hours: number;
  total_load: number;
};

type Fitness = {
  date: string;
  ctl: number;
  atl: number;
  tsb: number;
};

type Recovery = {
  hrv_7d: number | null;
  hrv_28d: number | null;
  hrv_vs_baseline_pct: number | null;
  resting_hr_7d: number | null;
  resting_hr_28d: number | null;
  sleep_hours_7d: number | null;
};

type ReadinessSignal = {
  name: string;
  level: string;
  detail: string;
};

type Readiness = {
  as_of: string;
  verdict: string;
  signals: ReadinessSignal[];
};

type DashboardData = {
  as_of: string;
  summary: {
    as_of: string;
    fitness: Fitness | null;
    recovery: Recovery;
    weeks: WeekSummary[];
  };
  readiness: Readiness;
};

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

const sportConfig = {
  bike: {
    name: "Cycling",
    color: "#D6FF5F",
    icon: Bike,
  },
  run: {
    name: "Running",
    color: "#79B9FF",
    icon: Footprints,
  },
  swim: {
    name: "Swimming",
    color: "#C19BFF",
    icon: Waves,
  },
  other: {
    name: "Other",
    color: "#929995",
    icon: Dumbbell,
  },
} as const;

function formatHours(hours: number): string {
  const totalMinutes = Math.round(hours * 60);
  const h = Math.floor(totalMinutes / 60);
  const m = totalMinutes % 60;

  if (h === 0) return `${m}m`;
  if (m === 0) return `${h}h`;

  return `${h}h ${m}m`;
}

function formatDistance(distance: number): string {
  return `${distance.toFixed(1)} km`;
}

function formatDate(date: string): string {
  const [year, month, day] = date.split("-").map(Number);

  return new Date(year, month - 1, day).toLocaleDateString("en-CA", {
    weekday: "long",
    month: "long",
    day: "numeric",
  });
}

function formatWeek(date: string): string {
  const [, month, day] = date.split("-").map(Number);

  return new Date(2000, month - 1, day).toLocaleDateString("en", {
    month: "short",
    day: "numeric",
  });
}

function getLatestWeek(weeks: WeekSummary[]): WeekSummary | undefined {
  return weeks.length > 0 ? weeks[weeks.length - 1] : undefined;
}

function getSportTotals(week: WeekSummary | undefined) {
  if (!week) return [];

  return (Object.keys(sportConfig) as Array<keyof typeof sportConfig>)
    .map((key) => {
      const stats = week.by_sport[key];

      return {
        key,
        ...sportConfig[key],
        hours: stats?.hours ?? 0,
      };
    })
    .filter((sport) => sport.hours > 0);
}

function getReadinessStyle(verdict: string) {
  switch (verdict.toLowerCase()) {
    case "go":
      return "bg-[#D6FF5F]/10 text-accent";
    case "caution":
      return "bg-yellow-400/10 text-yellow-400";
    case "rest":
    case "stop":
      return "bg-red-400/10 text-red-400";
    default:
      return "bg-[#292E32] text-muted";
  }
}

// -----------------------------------------------------------------------------
// Components
// -----------------------------------------------------------------------------

function Sidebar() {
  const navigation = [
    { label: "Overview", icon: LayoutDashboard, active: true },
    { label: "Activities", icon: Activity },
    { label: "Analytics", icon: ChartNoAxesCombined },
    { label: "Training", icon: CalendarDays },
  ];

  return (
    <aside className="fixed inset-y-0 left-0 z-20 hidden w-64 flex-col border-r border-border bg-[#151719] px-5 py-6 lg:flex">
      <div className="flex items-center gap-3 px-2">
        <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-accent text-[#111315]">
          <Activity size={21} strokeWidth={2.5} />
        </div>
        <span className="text-2xl font-bold tracking-tight">tempo</span>
      </div>

      <div className="mt-12">
        <p className="mb-4 px-3 text-[10px] font-semibold tracking-[0.18em] text-muted">
          WORKSPACE
        </p>

        <nav className="space-y-1">
          {navigation.map((item) => (
            <button
              key={item.label}
              className={`flex w-full items-center gap-3 rounded-lg px-3 py-3 text-sm transition ${
                item.active
                  ? "bg-[#292E32] font-medium text-accent"
                  : "text-[#A1A6A3] hover:bg-[#202427] hover:text-white"
              }`}
            >
              <item.icon size={18} />
              {item.label}
            </button>
          ))}
        </nav>
      </div>

      <div className="mt-auto space-y-2">
        <button className="flex w-full items-center gap-3 rounded-lg px-3 py-3 text-sm text-muted hover:bg-[#202427] hover:text-white">
          <Settings size={18} />
          Settings
        </button>

        <div className="flex items-center gap-3 border-t border-border px-2 pt-5">
          <div className="flex h-9 w-9 items-center justify-center rounded-full bg-[#343A3D]">
            <UserRound size={18} />
          </div>
          <div className="min-w-0 flex-1">
            <p className="truncate text-sm font-medium">Athlete</p>
            <p className="text-xs text-muted">Personal account</p>
          </div>
          <ChevronDown size={16} className="text-muted" />
        </div>
      </div>
    </aside>
  );
}

function MetricCard({
  label,
  value,
  description,
  icon: Icon,
}: {
  label: string;
  value: string;
  description: string;
  icon: typeof Activity;
}) {
  return (
    <div className="rounded-xl border border-border bg-surface p-5 transition hover:border-[#41484B]">
      <div className="flex items-center justify-between">
        <span className="text-sm text-muted">{label}</span>
        <Icon size={18} className="text-muted" />
      </div>

      <div className="mt-5">
        <h3 className="text-2xl font-semibold tracking-tight tabular-nums">
          {value}
        </h3>
      </div>

      <p className="mt-2 text-xs text-muted">{description}</p>
    </div>
  );
}

function TrainingChart({ weeks }: { weeks: WeekSummary[] }) {
  const chartData = weeks.map((week) => ({
    week: formatWeek(week.week_start),
    hours: week.total_hours,
  }));

  const average =
    weeks.length > 0
      ? weeks.reduce((sum, week) => sum + week.total_hours, 0) / weeks.length
      : 0;

  return (
    <section className="rounded-xl border border-border bg-surface p-5">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h2 className="font-semibold">Training volume</h2>
          <p className="mt-1 text-sm text-muted">
            Weekly training duration
          </p>
        </div>

        <span className="rounded-lg border border-border px-3 py-2 text-xs text-muted">
          Last {weeks.length} weeks
        </span>
      </div>

      <div className="mt-6 h-64 w-full">
        {chartData.length > 0 ? (
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={chartData}>
              <CartesianGrid
                stroke="#292E32"
                vertical={false}
                strokeDasharray="3 3"
              />
              <XAxis
                dataKey="week"
                axisLine={false}
                tickLine={false}
                tick={{ fill: "#929995", fontSize: 12 }}
                dy={10}
              />
              <YAxis
                axisLine={false}
                tickLine={false}
                tick={{ fill: "#929995", fontSize: 12 }}
                tickFormatter={(value) => `${value}h`}
                width={35}
              />
              <Tooltip
                cursor={{ fill: "#292E32", opacity: 0.5 }}
                contentStyle={{
                  background: "#202427",
                  border: "1px solid #41484B",
                  borderRadius: "8px",
                  color: "#F2F2EF",
                }}
                formatter={(value) => [
                  `${Number(value).toFixed(1)} hours`,
                  "Training",
                ]}
              />
              <Bar
                dataKey="hours"
                fill="#D6FF5F"
                radius={[5, 5, 0, 0]}
                maxBarSize={48}
              />
            </BarChart>
          </ResponsiveContainer>
        ) : (
          <div className="flex h-full items-center justify-center text-sm text-muted">
            No training history available.
          </div>
        )}
      </div>

      <div className="mt-4 flex items-center justify-between border-t border-border pt-4">
        <span className="text-xs text-muted">
          Average: {formatHours(average)} / week
        </span>
        <span className="text-xs text-muted">
          {weeks.length} weeks of data
        </span>
      </div>
    </section>
  );
}

function SportDistribution({ week }: { week?: WeekSummary }) {
  const sports = getSportTotals(week);
  const totalHours = sports.reduce((sum, sport) => sum + sport.hours, 0);

  return (
    <section className="rounded-xl border border-border bg-surface p-5">
      <h2 className="font-semibold">Sport distribution</h2>
      <p className="mt-1 text-sm text-muted">
        {week ? `Training time · Week of ${formatWeek(week.week_start)}` : "Training time by activity"}
      </p>

      {sports.length > 0 ? (
        <div className="mt-7 space-y-6">
          {sports.map((sport) => {
            const percentage =
              totalHours > 0 ? (sport.hours / totalHours) * 100 : 0;

            return (
              <div key={sport.key}>
                <div className="mb-2 flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <sport.icon size={15} style={{ color: sport.color }} />
                    <span className="text-sm">{sport.name}</span>
                  </div>
                  <span className="text-xs text-muted">
                    {formatHours(sport.hours)}
                  </span>
                </div>

                <div className="h-2 overflow-hidden rounded-full bg-[#292E32]">
                  <div
                    className="h-full rounded-full"
                    style={{
                      width: `${percentage}%`,
                      backgroundColor: sport.color,
                    }}
                  />
                </div>
              </div>
            );
          })}
        </div>
      ) : (
        <div className="mt-7 text-sm text-muted">
          No sport distribution available.
        </div>
      )}

      <div className="mt-7 border-t border-border pt-4">
        <div className="flex items-center justify-between">
          <span className="text-xs text-muted">Total training time</span>
          <span className="text-sm font-medium">
            {formatHours(totalHours)}
          </span>
        </div>
      </div>
    </section>
  );
}

function FitnessCard({ fitness }: { fitness: Fitness | null }) {
  return (
    <section className="rounded-xl border border-border bg-surface p-5">
      <div className="flex items-center justify-between">
        <h2 className="font-semibold">Fitness</h2>
        <Gauge size={18} className="text-muted" />
      </div>

      {fitness ? (
        <div className="mt-6 grid grid-cols-3 gap-3">
          <div>
            <p className="text-xs text-muted">Fitness</p>
            <p className="mt-2 text-2xl font-semibold tabular-nums">
              {fitness.ctl.toFixed(1)}
            </p>
            <p className="mt-1 text-[11px] text-muted">CTL</p>
          </div>
          <div>
            <p className="text-xs text-muted">Fatigue</p>
            <p className="mt-2 text-2xl font-semibold tabular-nums">
              {fitness.atl.toFixed(1)}
            </p>
            <p className="mt-1 text-[11px] text-muted">ATL</p>
          </div>
          <div>
            <p className="text-xs text-muted">Form</p>
            <p className="mt-2 text-2xl font-semibold tabular-nums text-accent">
              {fitness.tsb > 0 ? "+" : ""}
              {fitness.tsb.toFixed(1)}
            </p>
            <p className="mt-1 text-[11px] text-muted">TSB</p>
          </div>
        </div>
      ) : (
        <p className="mt-6 text-sm text-muted">No fitness data available.</p>
      )}
    </section>
  );
}


function RecoveryCard({ recovery }: { recovery: Recovery }) {
  const metrics = [
    {
      label: "Resting HR",
      value:
        recovery.resting_hr_7d !== null
          ? `${recovery.resting_hr_7d.toFixed(0)} bpm`
          : "—",
      description: "7-day average",
    },
    {
      label: "Sleep",
      value:
        recovery.sleep_hours_7d !== null
          ? `${recovery.sleep_hours_7d.toFixed(1)} h`
          : "—",
      description: "7-day average",
    },
  ];

  return (
    <section className="rounded-xl border border-border bg-surface p-5">
      <div className="flex items-center justify-between">
        <h2 className="font-semibold">Recovery</h2>
        <HeartPulse size={18} className="text-muted" />
      </div>

      <div className="mt-6 grid grid-cols-2 gap-4">
        {metrics.map((metric) => (
          <div key={metric.label}>
            <p className="text-xs text-muted">{metric.label}</p>
            <p className="mt-2 text-xl font-semibold tabular-nums">
              {metric.value}
            </p>
            <p className="mt-1 text-[11px] text-muted">
              {metric.description}
            </p>
          </div>
        ))}
      </div>
    </section>
  );
}


function ReadinessCard({ readiness }: { readiness: Readiness }) {
  return (
    <section className="rounded-xl border border-border bg-surface p-5">
      <div className="flex items-center justify-between">
        <h2 className="font-semibold">Daily readiness</h2>
        <Activity size={18} className="text-muted" />
      </div>

      <div className="mt-5">
        <span
          className={`inline-flex rounded-lg px-3 py-1.5 text-sm font-semibold capitalize ${getReadinessStyle(readiness.verdict)}`}
        >
          {readiness.verdict}
        </span>
      </div>

      <div className="mt-5 space-y-4">
        {readiness.signals.map((signal) => (
          <div key={signal.name} className="flex items-start gap-3">
            <span
              className={`mt-1.5 h-2 w-2 shrink-0 rounded-full ${
                signal.level === "ok"
                  ? "bg-accent"
                  : signal.level === "warning"
                    ? "bg-yellow-400"
                    : "bg-red-400"
              }`}
            />
            <div className="min-w-0">
              <p className="text-sm font-medium capitalize">
                {signal.name.replaceAll("_", " ")}
              </p>
              <p className="mt-1 text-xs leading-relaxed text-muted">
                {signal.detail}
              </p>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}

// -----------------------------------------------------------------------------
// Dashboard
// -----------------------------------------------------------------------------

function App() {
  const [data, setData] = useState<DashboardData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);

  useEffect(() => {
    const controller = new AbortController();

    async function fetchDashboard() {
      setLoading(true);
      setError(null);

      try {
        const response = await fetch("/api/dashboard?days=28", {
          signal: controller.signal,
        });

        if (!response.ok) {
          const message = await response.text();
          throw new Error(
            message || `Request failed with status ${response.status}`,
          );
        }

        const result: DashboardData = await response.json();
        setData(result);
      } catch (err) {
        if (err instanceof Error && err.name === "AbortError") return;

        setError(
          err instanceof Error ? err.message : "Unable to load dashboard data.",
        );
      } finally {
        if (!controller.signal.aborted) {
          setLoading(false);
        }
      }
    }

    fetchDashboard();

    return () => controller.abort();
  }, [refreshKey]);

  const latestWeek = data ? getLatestWeek(data.summary.weeks) : undefined;

  const totalSessions = latestWeek
    ? Object.values(latestWeek.by_sport).reduce(
        (sum, sport) => sum + (sport?.sessions ?? 0),
        0,
      )
    : 0;

  const totalDistance = latestWeek
    ? Object.values(latestWeek.by_sport).reduce(
        (sum, sport) => sum + (sport?.distance_km ?? 0),
        0,
      )
    : 0;

  const metrics = [
    {
      label: "Training time",
      value: latestWeek ? formatHours(latestWeek.total_hours) : "—",
      description: "Latest week",
      icon: Clock3,
    },
    {
      label: "Activities",
      value: latestWeek ? String(totalSessions) : "—",
      description: "Completed sessions · latest week",
      icon: Activity,
    },
    {
      label: "Distance",
      value: latestWeek ? formatDistance(totalDistance) : "—",
      description: "Latest week",
      icon: Route,
    },
    {
      label: "Training load",
      value: latestWeek ? String(Math.round(latestWeek.total_load)) : "—",
      description: "Weekly load",
      icon: ChartNoAxesCombined,
    },
  ];

  return (
    <div className="min-h-screen bg-background text-[#F2F2EF]">
      <Sidebar />

      <main className="min-h-screen lg:pl-64">
        <header className="flex h-16 items-center justify-between border-b border-border px-6 lg:px-10">
          <span className="text-sm text-muted">
            Personal performance / <span className="text-white">Overview</span>
          </span>

          <div className="flex items-center gap-4">
            <button
              onClick={() => setRefreshKey((key) => key + 1)}
              disabled={loading}
              title="Refresh dashboard"
              className="text-muted transition hover:text-white disabled:opacity-40"
            >
              <RefreshCw size={17} className={loading ? "animate-spin" : ""} />
            </button>
            <button className="text-muted transition hover:text-white">
              <Bell size={19} />
            </button>
            <div className="flex h-8 w-8 items-center justify-center rounded-full bg-[#343A3D]">
              <UserRound size={16} />
            </div>
          </div>
        </header>

        <div className="mx-auto max-w-[1500px] space-y-8 px-6 py-8 lg:px-10">
          <div className="flex flex-wrap items-end justify-between gap-4">
            <div>
              <p className="text-xs font-medium uppercase tracking-[0.15em] text-muted">
                {data ? formatDate(data.as_of) : "PERSONAL PERFORMANCE"}
              </p>
              <h1 className="mt-2 text-3xl font-semibold tracking-tight">
                Overview
              </h1>
              <p className="mt-2 text-sm text-muted">
                Here's how your training is progressing.
              </p>
            </div>

            <span className="rounded-lg border border-border bg-surface px-4 py-2.5 text-xs text-muted">
              Last 28 days
            </span>
          </div>

          {loading && !data ? (
            <div className="flex min-h-72 items-center justify-center gap-3 text-sm text-muted">
              <RefreshCw size={18} className="animate-spin" />
              Loading your training data...
            </div>
          ) : error && !data ? (
            <div className="rounded-xl border border-red-400/30 bg-red-400/5 p-6">
              <h2 className="font-semibold text-red-400">
                Unable to load dashboard
              </h2>
              <p className="mt-2 text-sm text-muted">{error}</p>
              <button
                onClick={() => setRefreshKey((key) => key + 1)}
                className="mt-5 rounded-lg border border-border px-4 py-2 text-sm hover:bg-[#292E32]"
              >
                Try again
              </button>
            </div>
          ) : data ? (
            <>
              {error && (
                <p className="text-xs text-yellow-400">
                  Refresh failed. Showing the last successfully loaded data.
                </p>
              )}

              <section className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
                {metrics.map((metric) => (
                  <MetricCard key={metric.label} {...metric} />
                ))}
              </section>

              <section className="grid grid-cols-1 gap-5 xl:grid-cols-3">
                <div className="xl:col-span-2">
                  <TrainingChart weeks={data.summary.weeks} />
                </div>
                <SportDistribution week={latestWeek} />
              </section>

              <section className="grid grid-cols-1 gap-5 xl:grid-cols-3">
                <FitnessCard fitness={data.summary.fitness} />
                <RecoveryCard recovery={data.summary.recovery} />
                <ReadinessCard readiness={data.readiness} />
              </section>
            </>
          ) : null}

          <footer className="border-t border-border py-6 text-center text-xs text-muted">
            TEMPO · PERSONAL PERFORMANCE TRACKING
          </footer>
        </div>
      </main>
    </div>
  );
}

export default App;
