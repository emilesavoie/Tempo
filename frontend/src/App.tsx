
import {
  Activity,
  Bike,
  CalendarDays,
  ChartNoAxesCombined,
  Clock3,
  Footprints,
  LayoutDashboard,
  Mountain,
  Settings,
  Waves,
  ArrowUpRight,
  Bell,
  ChevronDown,
  Route,
  UserRound,
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

const weeklyData = [
  { day: "Mon", hours: 1.2 },
  { day: "Tue", hours: 0.8 },
  { day: "Wed", hours: 1.5 },
  { day: "Thu", hours: 0.7 },
  { day: "Fri", hours: 0.5 },
  { day: "Sat", hours: 2.8 },
  { day: "Sun", hours: 1.2 },
];

const activities = [
  {
    name: "Endurance Ride",
    type: "Cycling",
    date: "Today, 9:30 AM",
    distance: "62.4 km",
    duration: "2h 10m",
    icon: Bike,
  },
  {
    name: "Easy Run",
    type: "Running",
    date: "Friday, 7:15 AM",
    distance: "8.2 km",
    duration: "48m",
    icon: Footprints,
  },
  {
    name: "Pool Session",
    type: "Swimming",
    date: "Thursday, 6:00 PM",
    distance: "2,000 m",
    duration: "42m",
    icon: Waves,
  },
];

const metrics = [
  {
    label: "Training time",
    value: "8h 42m",
    change: "+12%",
    icon: Clock3,
  },
  {
    label: "Activities",
    value: "7",
    change: "+2",
    icon: Activity,
  },
  {
    label: "Distance",
    value: "186.4 km",
    change: "+8%",
    icon: Route,
  },
  {
    label: "Elevation",
    value: "1,240 m",
    change: "+15%",
    icon: Mountain,
  },
];

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
  change,
  icon: Icon,
}: (typeof metrics)[number]) {
  return (
    <div className="rounded-xl border border-border bg-surface p-5 transition hover:border-[#41484B]">
      <div className="flex items-center justify-between">
        <span className="text-sm text-muted">{label}</span>
        <Icon size={18} className="text-muted" />
      </div>

      <div className="mt-5 flex items-end justify-between">
        <h3 className="text-2xl font-semibold tracking-tight tabular-nums">
          {value}
        </h3>
        <span className="mb-1 flex items-center gap-1 text-xs font-medium text-accent">
          <ArrowUpRight size={14} />
          {change}
        </span>
      </div>

      <p className="mt-2 text-xs text-muted">vs. previous week</p>
    </div>
  );
}

function TrainingChart() {
  return (
    <section className="rounded-xl border border-border bg-surface p-5">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h2 className="font-semibold">Training volume</h2>
          <p className="mt-1 text-sm text-muted">
            Daily training duration
          </p>
        </div>

        <button className="flex items-center gap-2 rounded-lg border border-border px-3 py-2 text-xs text-muted hover:text-white">
          This week
          <ChevronDown size={14} />
        </button>
      </div>

      <div className="mt-6 h-64 w-full">
        <ResponsiveContainer width="100%" height="100%">
          <BarChart data={weeklyData}>
            <CartesianGrid
              stroke="#292E32"
              vertical={false}
              strokeDasharray="3 3"
            />
            <XAxis
              dataKey="day"
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
              formatter={(value) => [`${value} hours`, "Training"]}
            />
            <Bar
              dataKey="hours"
              fill="#D6FF5F"
              radius={[5, 5, 0, 0]}
              maxBarSize={34}
            />
          </BarChart>
        </ResponsiveContainer>
      </div>

      <div className="mt-4 flex items-center justify-between border-t border-border pt-4">
        <span className="text-xs text-muted">
          Previous week: 7h 46m
        </span>
        <span className="text-xs font-medium text-accent">
          +12.0%
        </span>
      </div>
    </section>
  );
}

function SportDistribution() {
  const sports = [
    { name: "Cycling", time: "5h 10m", percent: 59, color: "#D6FF5F" },
    { name: "Running", time: "2h 02m", percent: 23, color: "#79B9FF" },
    { name: "Swimming", time: "1h 30m", percent: 18, color: "#C19BFF" },
  ];

  return (
    <section className="rounded-xl border border-border bg-surface p-5">
      <h2 className="font-semibold">Sport distribution</h2>
      <p className="mt-1 text-sm text-muted">Training time by activity</p>

      <div className="mt-7 space-y-6">
        {sports.map((sport) => (
          <div key={sport.name}>
            <div className="mb-2 flex items-center justify-between">
              <span className="text-sm">{sport.name}</span>
              <span className="text-xs text-muted">{sport.time}</span>
            </div>

            <div className="h-2 overflow-hidden rounded-full bg-[#292E32]">
              <div
                className="h-full rounded-full"
                style={{
                  width: `${sport.percent}%`,
                  backgroundColor: sport.color,
                }}
              />
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}

function RecentActivities() {
  return (
    <section className="rounded-xl border border-border bg-surface p-5">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="font-semibold">Recent activities</h2>
          <p className="mt-1 text-sm text-muted">
            Your latest training sessions
          </p>
        </div>

        <button className="text-xs font-medium text-accent hover:underline">
          View all
        </button>
      </div>

      <div className="mt-5 divide-y divide-border">
        {activities.map((activity) => (
          <div
            key={activity.name}
            className="flex items-center gap-4 py-4 first:pt-0 last:pb-0"
          >
            <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-[#292E32] text-accent">
              <activity.icon size={19} />
            </div>

            <div className="min-w-0 flex-1">
              <p className="truncate text-sm font-medium">
                {activity.name}
              </p>
              <p className="mt-1 text-xs text-muted">
                {activity.type} · {activity.date}
              </p>
            </div>

            <div className="shrink-0 text-right">
              <p className="text-sm font-medium tabular-nums">
                {activity.distance}
              </p>
              <p className="mt-1 text-xs text-muted">
                {activity.duration}
              </p>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}

function App() {
  return (
    <div className="min-h-screen bg-background text-[#F2F2EF]">
      <Sidebar />

      <main className="min-h-screen lg:pl-64">
        <header className="flex h-16 items-center justify-between border-b border-border px-6 lg:px-10">
          <span className="text-sm text-muted">
            Personal performance / <span className="text-white">Overview</span>
          </span>

          <div className="flex items-center gap-4">
            <button className="text-muted transition hover:text-white">
              <Bell size={19} />
            </button>
            <div className="h-8 w-8 rounded-full bg-[#343A3D] flex items-center justify-center">
              <UserRound size={16} />
            </div>
          </div>
        </header>

        <div className="mx-auto max-w-[1500px] space-y-8 px-6 py-8 lg:px-10">
          <div className="flex flex-wrap items-end justify-between gap-4">
            <div>
              <p className="text-xs font-medium uppercase tracking-[0.15em] text-muted">
                Sunday, November 15
              </p>
              <h1 className="mt-2 text-3xl font-semibold tracking-tight">
                Overview
              </h1>
              <p className="mt-2 text-sm text-muted">
                Here's how your training is progressing.
              </p>
            </div>

            <button className="flex items-center gap-2 rounded-lg border border-border bg-surface px-4 py-2.5 text-sm hover:border-[#41484B]">
              This week
              <ChevronDown size={16} />
            </button>
          </div>

          <section className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
            {metrics.map((metric) => (
              <MetricCard key={metric.label} {...metric} />
            ))}
          </section>

          <section className="grid grid-cols-1 gap-5 xl:grid-cols-3">
            <div className="xl:col-span-2">
              <TrainingChart />
            </div>
            <SportDistribution />
          </section>

          <RecentActivities />

          <footer className="border-t border-border py-6 text-center text-xs text-muted">
            TEMPO · PERSONAL PERFORMANCE TRACKING
          </footer>
        </div>
      </main>
    </div>
  );
}

export default App;
