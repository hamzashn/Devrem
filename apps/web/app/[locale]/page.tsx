import Link from "next/link";
import { CircuitMotif } from "@/components/CircuitMotif";
import { Screenshot } from "@/components/Screenshot";
import { getDictionary } from "@/lib/i18n";

const FEATURE_ICONS = ["⚡", "⇄", "∿", "⚙", "✎", "⇪"];

export default async function Home({ params }: { params: Promise<{ locale: string }> }) {
  const { locale } = await params;
  const t = getDictionary(locale);

  return (
    <>
      {/* ------------------------------------------------------------ Giriş */}
      <section className="grid-dots relative overflow-hidden border-b border-border">
        <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_top,rgba(34,211,238,0.12),transparent_60%)]" />
        <CircuitMotif className="pointer-events-none absolute -top-10 right-0 hidden w-[520px] opacity-40 lg:block" />
        <div className="relative mx-auto max-w-6xl px-5 pt-20 pb-16 text-center">
          <span className="inline-block rounded-full border border-accent/30 bg-accent/10 px-3 py-1 font-mono text-xs text-accent">
            {t.hero.badge}
          </span>
          <h1 className="mx-auto mt-6 max-w-3xl text-4xl font-bold tracking-tight sm:text-6xl">
            {t.hero.title}{" "}
            <span className="bg-gradient-to-r from-accent to-[var(--current)] bg-clip-text text-transparent">
              {t.hero.titleAccent}
            </span>
          </h1>
          <p className="mx-auto mt-6 max-w-2xl text-lg leading-relaxed text-secondary">
            {t.hero.subtitle}
          </p>
          <div className="mt-9 flex flex-wrap justify-center gap-3">
            <Link
              href={`/${locale}/download/`}
              className="rounded-lg bg-accent px-6 py-3 font-semibold text-[var(--bg-base)] shadow-[0_0_30px_var(--accent-glow)] transition hover:brightness-110"
            >
              {t.hero.primary}
            </Link>
            <a
              href="#features"
              className="rounded-lg border border-border bg-surface px-6 py-3 font-medium text-primary transition hover:border-accent/50"
            >
              {t.hero.secondary}
            </a>
          </div>
          <Screenshot
            src="/screenshots/hero.png"
            alt={t.showcase.scopeText}
            priority
            className="mx-auto mt-16 max-w-5xl"
          />
        </div>
      </section>

      {/* ------------------------------------------------------- Özellikler */}
      <section id="features" className="mx-auto max-w-6xl scroll-mt-20 px-5 pt-24">
        <h2 className="text-center text-3xl font-bold tracking-tight">{t.features.title}</h2>
        <p className="mx-auto mt-3 max-w-2xl text-center text-secondary">{t.features.subtitle}</p>
        <div className="mt-12 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {t.features.items.map((f, i) => (
            <article
              key={f.title}
              className="rounded-xl border border-border bg-surface p-6 transition hover:border-accent/40"
            >
              <div className="flex size-10 items-center justify-center rounded-lg bg-accent/10 text-lg text-accent">
                {FEATURE_ICONS[i]}
              </div>
              <h3 className="mt-4 font-semibold">{f.title}</h3>
              <p className="mt-2 text-sm leading-relaxed text-secondary">{f.text}</p>
            </article>
          ))}
        </div>
      </section>

      {/* ------------------------------------------------------- Vitrin */}
      <section className="mx-auto max-w-6xl space-y-20 px-5 pt-24">
        {[
          {
            src: "/screenshots/scope.png",
            title: t.showcase.scopeTitle,
            text: t.showcase.scopeText,
          },
          {
            src: "/screenshots/digital.png",
            title: t.showcase.digitalTitle,
            text: t.showcase.digitalText,
          },
          { src: "/screenshots/bode.png", title: t.showcase.acTitle, text: t.showcase.acText },
        ].map((item, i) => (
          <div
            key={item.src}
            className={`grid items-center gap-10 lg:grid-cols-5 ${i % 2 ? "lg:[&>div:first-child]:order-2" : ""}`}
          >
            <div className="lg:col-span-2">
              <h3 className="text-2xl font-bold tracking-tight">{item.title}</h3>
              <p className="mt-3 leading-relaxed text-secondary">{item.text}</p>
            </div>
            <Screenshot src={item.src} alt={item.text} className="lg:col-span-3" />
          </div>
        ))}
      </section>

      {/* ------------------------------------------------------- Bileşenler */}
      <section className="mx-auto max-w-6xl px-5 pt-24">
        <h2 className="text-center text-3xl font-bold tracking-tight">{t.components.title}</h2>
        <div className="mt-10 grid gap-px overflow-hidden rounded-xl border border-border bg-border sm:grid-cols-2 lg:grid-cols-4">
          {t.components.groups.map((g) => (
            <div key={g.name} className="bg-surface p-5">
              <h3 className="font-mono text-xs tracking-wider text-accent uppercase">{g.name}</h3>
              <p className="mt-2 text-sm leading-relaxed text-secondary">{g.items}</p>
            </div>
          ))}
        </div>
      </section>

      {/* ------------------------------------------------------- Çağrı */}
      <section className="mx-auto max-w-6xl px-5 pt-24">
        <div className="grid-dots relative overflow-hidden rounded-2xl border border-accent/25 bg-surface px-8 py-14 text-center">
          <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_center,rgba(34,211,238,0.1),transparent_70%)]" />
          <h2 className="relative text-3xl font-bold tracking-tight">{t.cta.title}</h2>
          <p className="relative mx-auto mt-3 max-w-xl text-secondary">{t.cta.text}</p>
          <Link
            href={`/${locale}/download/`}
            className="relative mt-8 inline-block rounded-lg bg-accent px-6 py-3 font-semibold text-[var(--bg-base)] shadow-[0_0_30px_var(--accent-glow)] transition hover:brightness-110"
          >
            {t.cta.button}
          </Link>
        </div>
      </section>
    </>
  );
}
