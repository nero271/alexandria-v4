import { useEffect, useState } from "react";
import { Icon } from "../../components/Icon";
import { Modal } from "../../components/Modal";
import { asset } from "../../lib/assets";
import { fmtPlaytime } from "../../lib/format";
import { api, errorMessage } from "../../lib/ipc";
import type { Game, YearStats } from "../../lib/types";

const hours = (min: number) => fmtPlaytime(min) || "0 min";

/** Resumo do ano a partir das sessões registradas pelo ALEXANDRIA. */
export function YearModal({ games, onClose }: { games: Game[]; onClose: () => void }) {
  const [year, setYear] = useState(new Date().getFullYear());
  const [st, setSt] = useState<YearStats | null>(null);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    api
      .yearStats(year, -new Date().getTimezoneOffset())
      .then((s) => alive && setSt(s))
      .catch((e) => alive && setErr(errorMessage(e)));
    return () => {
      alive = false;
    };
  }, [year]);

  return (
    <Modal title={<Icon name="chart" size={24} />} label="Resumo do ano" onClose={onClose} wide>
      {err && <p className="form-error">{err}</p>}
      {st && (
        <>
          <div className="chips">
            {st.years.map((y) => (
              <button key={y} className="btn" aria-pressed={y === year} onClick={() => setYear(y)}>{y}</button>
            ))}
          </div>
          <div className="year-grid">
            <div className="year-card"><b>{hours(st.minutes)}</b><span>jogadas</span></div>
            <div className="year-card"><b>{st.sessions}</b><span>sessões</span></div>
            <div className="year-card"><b>{st.days}</b><span>dias jogados</span></div>
            <div className="year-card"><b>{st.longestStreak}</b><span>maior sequência (dias)</span></div>
          </div>
          <p className="muted">
            Sequência atual: {st.currentStreak} dia(s) · Tempo total na biblioteca: {hours(st.lifetimeMinutes)} em {st.gamesTotal} jogos
          </p>
          <h3>Mais jogados em {year}</h3>
          {st.top.length === 0 ? (
            <p className="muted">Ainda não há sessões registradas neste ano.</p>
          ) : (
            <ol className="year-top">
              {st.top.map((t) => {
                const g = games.find((x) => x.id === t.id);
                const cover = asset(g?.coverPath);
                return (
                  <li key={t.id}>
                    {cover ? <img src={cover} alt="" /> : <div className="ph" />}
                    <span className="name">{t.name}</span>
                    <span className="muted">{hours(t.minutes)} · {t.sessions} sessão(ões)</span>
                  </li>
                );
              })}
            </ol>
          )}
          <p className="muted">
            Conta apenas as sessões que o ALEXANDRIA registrou a partir desta versão. O tempo total de cada jogo continua vindo da Steam.
          </p>
        </>
      )}
      <div className="acts"><button className="btn primary" onClick={onClose}>Fechar</button></div>
    </Modal>
  );
}
