// Charlie Tanfoll Pereira Lobo - 16827968
// Marina Cintra Queiroz - 17074404

//! # Módulo de Visualização Cartográfica (Leaflet.js)
//!
//! Substitui o script `verRota.py` por uma implementação nativa em Rust que gera uma página
//! HTML interativa com a biblioteca **Leaflet.js** sobre o mapa do OpenStreetMap.
//!
//! ## Fluxo de dados (respeitando a estrutura do [`Grafo`])
//!
//! Nenhum arquivo `.csv` é relido. Cada rota é desenhada exclusivamente a partir do vetor de
//! `osmid` devolvido pelas buscas ([`ResultadoBusca::caminho`]):
//!
//! ```text
//! osmid ──osmid_para_id()──► NodeId ──coordenada()──► Coordenada (UTM 22S, metros)
//!       ──para_geografica()──► CoordenadaGeo (latitude, longitude) ──► L.polyline
//! ```
//!
//! Como o grafo armazena apenas as coordenadas planas UTM (SIRGAS 2000 / 22S, EPSG:31982),
//! usadas na heurística $h(n)$, a latitude/longitude exigida pelo Leaflet é obtida pela
//! **projeção UTM inversa** (série de Snyder), a operação inversa da realizada em `data/transforma.py`.
//!
//! ## Customizações visuais
//! - **A\* (Desafio 1 - Rota Ótima)**: linha **roxa** (`#8A2BE2`), 7 px.
//! - **GBFS (Desafio 2 - Rota Expressa)**: linha **vermelha** (`#E53935`), 4 px, desenhada por cima
//!   do A\*; em trechos coincidentes a linha vermelha corre pelo centro da roxa e ambas ficam visíveis.
//! - **Controle de camadas** para ligar/desligar cada rota.
//! - **Painel flutuante** com distância, cruzamentos, nós expandidos e variação percentual do GBFS.
//! - **Enquadramento automático** (`fitBounds`) e **abertura opcional no navegador**.

use std::fmt::Write as _;
use std::fs;
use std::io;

use crate::graph::Grafo;
use crate::structs::{Coordenada, ResultadoBusca};

// ---------------------------------------------------------------------------------------------
// Parâmetros geodésicos: elipsoide GRS80 (SIRGAS 2000) + projeção UTM zona 22 Sul (EPSG:31982)
// ---------------------------------------------------------------------------------------------

/// Semi-eixo maior do elipsoide GRS80, em metros.
const SEMI_EIXO_MAIOR: f64 = 6_378_137.0;
/// Achatamento do elipsoide GRS80.
const ACHATAMENTO: f64 = 1.0 / 298.257_222_101;
/// Fator de escala no meridiano central da projeção UTM.
const FATOR_ESCALA_K0: f64 = 0.9996;
/// Falso Este aplicado a todas as zonas UTM, em metros.
const FALSO_ESTE: f64 = 500_000.0;
/// Falso Norte aplicado no hemisfério Sul, em metros.
const FALSO_NORTE_SUL: f64 = 10_000_000.0;
/// Meridiano central da zona UTM 22 (−51°).
const MERIDIANO_CENTRAL_GRAUS: f64 = -51.0;

// ---------------------------------------------------------------------------------------------
// Identidade visual das rotas
// ---------------------------------------------------------------------------------------------

/// Cor da rota do A* (roxo).
const COR_A_STAR: &str = "#8A2BE2";
/// Cor de fundo do card do A* no painel.
const FUNDO_A_STAR: &str = "#F3E5F5";
/// Cor da rota do GBFS (vermelho).
const COR_GBFS: &str = "#E53935";
/// Cor de fundo do card do GBFS no painel.
const FUNDO_GBFS: &str = "#FFEBEE";

/// Posição geográfica em graus decimais (datum SIRGAS 2000), formato exigido pelo Leaflet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoordenadaGeo {
    /// Latitude em graus decimais (negativa no hemisfério Sul).
    pub latitude: f64,
    /// Longitude em graus decimais (negativa a oeste de Greenwich).
    pub longitude: f64,
}

impl Coordenada {
    /// Converte a coordenada plana UTM (SIRGAS 2000 / 22S) em latitude/longitude.
    ///
    /// Implementa a projeção Transversa de Mercator inversa pelas séries de Snyder
    /// (*Map Projections: A Working Manual*, USGS 1987, eq. 8-18 a 8-25), com precisão
    /// submétrica em toda a extensão da zona — mais do que suficiente para visualização.
    pub fn para_geografica(&self) -> CoordenadaGeo {
        let e2 = ACHATAMENTO * (2.0 - ACHATAMENTO); // excentricidade ao quadrado
        let ep2 = e2 / (1.0 - e2); // segunda excentricidade ao quadrado

        let x = self.x - FALSO_ESTE;
        let y = self.y - FALSO_NORTE_SUL;

        // Latitude do ponto-pé (footpoint latitude) phi1
        let arco_meridiano = y / FATOR_ESCALA_K0;
        let mu = arco_meridiano
            / (SEMI_EIXO_MAIOR
                * (1.0 - e2 / 4.0 - 3.0 * e2.powi(2) / 64.0 - 5.0 * e2.powi(3) / 256.0));

        let raiz = (1.0 - e2).sqrt();
        let e1 = (1.0 - raiz) / (1.0 + raiz);

        let phi1 = mu
            + (3.0 * e1 / 2.0 - 27.0 * e1.powi(3) / 32.0) * (2.0 * mu).sin()
            + (21.0 * e1.powi(2) / 16.0 - 55.0 * e1.powi(4) / 32.0) * (4.0 * mu).sin()
            + (151.0 * e1.powi(3) / 96.0) * (6.0 * mu).sin()
            + (1097.0 * e1.powi(4) / 512.0) * (8.0 * mu).sin();

        let (sen_phi1, cos_phi1) = phi1.sin_cos();
        let tan_phi1 = sen_phi1 / cos_phi1;

        let c1 = ep2 * cos_phi1 * cos_phi1;
        let t1 = tan_phi1 * tan_phi1;
        let w = 1.0 - e2 * sen_phi1 * sen_phi1;
        let n1 = SEMI_EIXO_MAIOR / w.sqrt(); // raio de curvatura no primeiro vertical
        let r1 = SEMI_EIXO_MAIOR * (1.0 - e2) / (w * w.sqrt()); // raio de curvatura meridiano
        let d = x / (n1 * FATOR_ESCALA_K0);

        let (d2, d3, d4, d5, d6) = (d.powi(2), d.powi(3), d.powi(4), d.powi(5), d.powi(6));

        let latitude = phi1
            - (n1 * tan_phi1 / r1)
                * (d2 / 2.0
                    - (5.0 + 3.0 * t1 + 10.0 * c1 - 4.0 * c1 * c1 - 9.0 * ep2) * d4 / 24.0
                    + (61.0 + 90.0 * t1 + 298.0 * c1 + 45.0 * t1 * t1 - 252.0 * ep2 - 3.0 * c1 * c1)
                        * d6
                        / 720.0);

        let longitude = MERIDIANO_CENTRAL_GRAUS.to_radians()
            + (d - (1.0 + 2.0 * t1 + c1) * d3 / 6.0
                + (5.0 - 2.0 * c1 + 28.0 * t1 - 3.0 * c1 * c1 + 8.0 * ep2 + 24.0 * t1 * t1) * d5
                    / 120.0)
                / cos_phi1;

        CoordenadaGeo {
            latitude: latitude.to_degrees(),
            longitude: longitude.to_degrees(),
        }
    }
}

impl Grafo {
    /// Retorna a posição geográfica de um cruzamento a partir do seu `osmid`.
    ///
    /// Percorre a mesma cadeia usada pelas buscas: `osmid → NodeId → Coordenada (UTM)`,
    /// aplicando ao final a projeção inversa para latitude/longitude.
    #[inline]
    pub fn coordenada_geografica(&self, osmid: u64) -> Option<CoordenadaGeo> {
        self.osmid_para_id(osmid)
            .map(|id| self.coordenada(id).para_geografica())
    }

    /// Converte o vetor ordenado de `osmid` de uma rota na sequência de pontos geográficos
    /// que compõem a polilinha desenhada no mapa.
    pub fn caminho_geografico(&self, caminho_osmid: &[u64]) -> Vec<CoordenadaGeo> {
        caminho_osmid
            .iter()
            .filter_map(|&osmid| self.coordenada_geografica(osmid))
            .collect()
    }

    /// Gera a página HTML interativa com as rotas encontradas pelo A* (roxo) e pelo GBFS (vermelho).
    ///
    /// # Parâmetros
    ///
    /// * `origem_osmid` / `destino_osmid` - Extremidades da busca (marcadores verde e vermelho).
    /// * `resultado_a_star` - Resultado do Desafio 1, se houver.
    /// * `resultado_gbfs` - Resultado do Desafio 2, se houver (comparado percentualmente ao A*).
    /// * `caminho_saida` - Arquivo HTML a ser gravado (ex.: `"mapa_rotas.html"`).
    /// * `abrir_navegador` - Se `true`, abre o arquivo no navegador padrão após gravá-lo.
    ///
    /// # Errors
    ///
    /// Retorna [`io::ErrorKind::InvalidInput`] se a origem ou o destino não pertencerem à malha,
    /// ou o erro de E/S correspondente caso a gravação do arquivo falhe.
    pub fn gerar_mapa_rotas(
        &self,
        origem_osmid: u64,
        destino_osmid: u64,
        resultado_a_star: Option<&ResultadoBusca>,
        resultado_gbfs: Option<&ResultadoBusca>,
        caminho_saida: &str,
        abrir_navegador: bool,
    ) -> io::Result<()> {
        let nao_encontrado = |osmid: u64| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("OSMID {osmid} não pertence à malha viária de Ribeirão Preto"),
            )
        };

        let ponto_origem = self
            .coordenada_geografica(origem_osmid)
            .ok_or_else(|| nao_encontrado(origem_osmid))?;
        let ponto_destino = self
            .coordenada_geografica(destino_osmid)
            .ok_or_else(|| nao_encontrado(destino_osmid))?;

        // Polilinhas geradas diretamente dos vetores de osmid de cada busca
        let rota_a_star = resultado_a_star
            .map(|res| self.caminho_geografico(&res.caminho))
            .unwrap_or_default();
        let rota_gbfs = resultado_gbfs
            .map(|res| self.caminho_geografico(&res.caminho))
            .unwrap_or_default();

        // Cards de métricas do painel flutuante
        let card_a_star = resultado_a_star
            .map(|res| {
                card_algoritmo("🟣 A* (Rota Ótima)", "Desafio 1", COR_A_STAR, FUNDO_A_STAR, res, None)
            })
            .unwrap_or_default();
        let card_gbfs = resultado_gbfs
            .map(|res| {
                card_algoritmo(
                    "🔴 GBFS (Rota Expressa)",
                    "Desafio 2",
                    COR_GBFS,
                    FUNDO_GBFS,
                    res,
                    resultado_a_star,
                )
            })
            .unwrap_or_default();

        let html = TEMPLATE_HTML
            .replace("__ORIGEM__", &origem_osmid.to_string())
            .replace("__DESTINO__", &destino_osmid.to_string())
            .replace("__CARD_A_STAR__", &card_a_star)
            .replace("__CARD_GBFS__", &card_gbfs)
            .replace("__ROTA_A_STAR__", &polilinha_js(&rota_a_star))
            .replace("__ROTA_GBFS__", &polilinha_js(&rota_gbfs))
            .replace("__PONTO_ORIGEM__", &ponto_js(ponto_origem))
            .replace("__PONTO_DESTINO__", &ponto_js(ponto_destino))
            .replace("__COR_A_STAR__", COR_A_STAR)
            .replace("__COR_GBFS__", COR_GBFS);

        fs::write(caminho_saida, html)?;

        if abrir_navegador {
            abrir_no_navegador(caminho_saida)?;
        }

        Ok(())
    }
}

/// Abre um arquivo no navegador web padrão do sistema operacional.
pub fn abrir_no_navegador(caminho_arquivo: &str) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    let mut comando = {
        let mut cmd = std::process::Command::new("cmd");
        cmd.args(["/C", "start", "", caminho_arquivo]);
        cmd
    };

    #[cfg(target_os = "macos")]
    let mut comando = {
        let mut cmd = std::process::Command::new("open");
        cmd.arg(caminho_arquivo);
        cmd
    };

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let mut comando = {
        let mut cmd = std::process::Command::new("xdg-open");
        cmd.arg(caminho_arquivo);
        cmd
    };

    comando.spawn().map(|_| ())
}

/// Serializa um ponto geográfico como array JavaScript `[lat, lon]`.
#[inline]
fn ponto_js(ponto: CoordenadaGeo) -> String {
    format!("[{:.7}, {:.7}]", ponto.latitude, ponto.longitude)
}

/// Serializa a sequência de pontos de uma rota como array JavaScript `[[lat, lon], ...]`.
fn polilinha_js(pontos: &[CoordenadaGeo]) -> String {
    let mut js = String::with_capacity(pontos.len() * 28 + 2);
    js.push('[');
    for (i, ponto) in pontos.iter().enumerate() {
        if i > 0 {
            js.push_str(", ");
        }
        let _ = write!(js, "[{:.7}, {:.7}]", ponto.latitude, ponto.longitude);
    }
    js.push(']');
    js
}

/// Variação percentual de `valor` em relação a `referencia`, formatada como ` (+X.XX%)`.
fn variacao_percentual(valor: f64, referencia: f64) -> String {
    if referencia == 0.0 {
        return String::new();
    }
    format!(" ({:+.2}%)", (valor - referencia) / referencia * 100.0)
}

/// Monta o card HTML de métricas de um algoritmo para o painel flutuante.
///
/// Se `referencia` for informada (A*), exibe a variação percentual de distância e de nós expandidos.
fn card_algoritmo(
    titulo: &str,
    selo: &str,
    cor: &str,
    fundo: &str,
    resultado: &ResultadoBusca,
    referencia: Option<&ResultadoBusca>,
) -> String {
    let (var_distancia, var_expandidos) = match referencia {
        Some(ref_) => (
            variacao_percentual(resultado.distancia_total, ref_.distancia_total),
            variacao_percentual(resultado.nos_expandidos as f64, ref_.nos_expandidos as f64),
        ),
        None => (String::new(), String::new()),
    };

    format!(
        r#"<div class="card" style="--cor: {cor}; --fundo: {fundo};">
            <div class="card-titulo"><span>{titulo}</span><span class="selo">{selo}</span></div>
            <div class="card-linha"><b>Distância:</b> {distancia:.2} m{var_distancia}</div>
            <div class="card-linha"><b>Cruzamentos:</b> {cruzamentos}</div>
            <div class="card-linha"><b>Nós expandidos:</b> {expandidos}{var_expandidos}</div>
        </div>"#,
        distancia = resultado.distancia_total,
        cruzamentos = resultado.caminho.len(),
        expandidos = resultado.nos_expandidos,
    )
}

/// Modelo da página HTML. Os marcadores `__NOME__` são substituídos em [`Grafo::gerar_mapa_rotas`].
const TEMPLATE_HTML: &str = r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rotas de Ribeirão Preto - A* vs GBFS</title>
    <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
          integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY=" crossorigin="" />
    <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"
            integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo=" crossorigin=""></script>
    <style>
        html, body { height: 100%; margin: 0; padding: 0;
                     font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Arial, sans-serif; }
        #map { width: 100%; height: 100%; }
        .painel { position: absolute; bottom: 24px; left: 16px; z-index: 1000; width: 300px;
                  max-width: calc(100vw - 64px); padding: 14px 18px; border-radius: 12px;
                  background: rgba(255, 255, 255, 0.96); box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2); }
        .painel-titulo { font-size: 15px; font-weight: 800; color: #1a202c; }
        .painel-subtitulo { font-size: 11px; color: #718096; margin-bottom: 10px; }
        .extremos { font-size: 12px; background: #f7fafc; padding: 8px 10px; border-radius: 6px;
                    border: 1px solid #e2e8f0; }
        .card { margin-top: 10px; padding: 10px; border-radius: 6px;
                background: var(--fundo); border-left: 4px solid var(--cor); }
        .card-titulo { display: flex; justify-content: space-between; align-items: center;
                       font-weight: 700; color: var(--cor); }
        .selo { font-size: 11px; padding: 2px 6px; border-radius: 4px; background: rgba(0, 0, 0, 0.06); }
        .card-linha { font-size: 12px; margin-top: 4px; color: #333; }
    </style>
</head>
<body>
    <div id="map"></div>

    <div class="painel">
        <div class="painel-titulo">Roteamento de Ribeirão Preto</div>
        <div class="painel-subtitulo">Inteligência Artificial 2026 • A* vs GBFS</div>
        <div class="extremos">
            <div>🟢 <b>Origem:</b> __ORIGEM__</div>
            <div>🏁 <b>Destino:</b> __DESTINO__</div>
        </div>
        __CARD_A_STAR__
        __CARD_GBFS__
    </div>

    <script>
        var map = L.map('map').setView(__PONTO_ORIGEM__, 13);

        L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
            maxZoom: 19,
            attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
        }).addTo(map);

        var rotaAStar = __ROTA_A_STAR__;
        var rotaGBFS = __ROTA_GBFS__;

        var camadas = {};
        var enquadramento = L.featureGroup();

        // A* em roxo, mais largo: serve de "trilho" visível sob o GBFS em trechos coincidentes
        if (rotaAStar.length > 0) {
            var linhaAStar = L.polyline(rotaAStar, {
                color: '__COR_A_STAR__', weight: 7, opacity: 0.85, lineJoin: 'round'
            }).bindPopup('<b>🟣 Rota Ótima (A*)</b><br>Desafio 1').addTo(map);
            enquadramento.addLayer(linhaAStar);
            camadas['🟣 A* (Rota Ótima)'] = linhaAStar;
        }

        // GBFS em vermelho, mais fino e desenhado por cima
        if (rotaGBFS.length > 0) {
            var linhaGBFS = L.polyline(rotaGBFS, {
                color: '__COR_GBFS__', weight: 4, opacity: 0.95, lineJoin: 'round'
            }).bindPopup('<b>🔴 Rota Expressa (GBFS)</b><br>Desafio 2').addTo(map);
            enquadramento.addLayer(linhaGBFS);
            camadas['🔴 GBFS (Rota Expressa)'] = linhaGBFS;
        }

        var estiloMarcador = { radius: 8, color: '#FFFFFF', weight: 3, opacity: 1, fillOpacity: 0.95 };

        L.circleMarker(__PONTO_ORIGEM__, Object.assign({ fillColor: '#2E7D32' }, estiloMarcador))
            .bindPopup('<b>🟢 Origem</b><br>OSMID: __ORIGEM__')
            .addTo(enquadramento);
        L.circleMarker(__PONTO_DESTINO__, Object.assign({ fillColor: '#C62828' }, estiloMarcador))
            .bindPopup('<b>🏁 Destino</b><br>OSMID: __DESTINO__')
            .addTo(enquadramento);

        enquadramento.addTo(map);
        L.control.layers(null, camadas, { collapsed: false }).addTo(map);
        map.fitBounds(enquadramento.getBounds(), { padding: [60, 60] });
    </script>
</body>
</html>
"#;

#[cfg(test)]
mod testes {
    use super::*;

    /// Tolerância de 1e-6 grau (~0,1 m), comparada aos valores originais de `trabalho/nos.csv`.
    const TOLERANCIA_GRAUS: f64 = 1e-6;

    fn verificar(x: f64, y: f64, lat_esperada: f64, lon_esperada: f64) {
        let geo = Coordenada::new(x, y).para_geografica();
        assert!(
            (geo.latitude - lat_esperada).abs() < TOLERANCIA_GRAUS,
            "latitude {} != {}",
            geo.latitude,
            lat_esperada
        );
        assert!(
            (geo.longitude - lon_esperada).abs() < TOLERANCIA_GRAUS,
            "longitude {} != {}",
            geo.longitude,
            lon_esperada
        );
    }

    #[test]
    fn utm_inversa_reproduz_nos_csv() {
        // osmid 259576101 (UTM de data/coordenadas_finais.csv, lat/lon de trabalho/nos.csv)
        verificar(836391.6495, 7648443.2593, -21.2345722, -47.759441);
        // osmid 259577028
        verificar(836145.281, 7659389.0953, -21.1358758, -47.7639634);
    }
}
