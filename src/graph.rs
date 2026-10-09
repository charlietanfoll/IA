// Charlie Tanfoll Pereira Lobo - 16827968
// Marina Cintra Queiroz - 17074404

//! # Módulo de Gerenciamento e Serialização do Grafo
//!
//! Este módulo é responsável pelo ciclo de vida do grafo:
//! - Parse dos arquivos de dados tabulares (.csv) da pasta `data/`.
//! - Construção da representação comprimida CSR (*Compressed Sparse Row*) bidirecional.
//! - Serialização e desserialização binária de alta velocidade (`rp.graph`) com cabeçalho seguro
//!   contendo *Magic Number*, versão, tamanho de metadados e byte sentinela.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

use crate::structs::*;

/// Assinatura mágica de 4 bytes (*Magic Number*) em formato ASCII: `"RPGR"` (*Ribeirão Preto Graph*).
///
/// Posicionada nos bytes `0..4` do arquivo para identificação e validação do formato binário.
pub const MAGIC_BYTES: &[u8; 4] = b"RPGR";

/// Versão do layout de serialização binária do grafo.
///
/// Versão 2: arquitetura SoA (*Structure of Arrays*) com arrays de destinos e distâncias separados,
/// eliminando 100% de padding e permitindo leitura/escrita de blocos contíguos de memória.
pub const FORMAT_VERSION: u8 = 2;

/// Byte sentinela gravado no término exato do cabeçalho de metadados (`0xFF`).
///
/// Utilizado como *checkpoint* de integridade para garantir que o cursor de leitura
/// está perfeitamente alinhado antes de iniciar a alocação de memória dos blocos de dados.
pub const HEADER_MARKER: u8 = 0xFF;

/// Escreve uma fatia contígua de dados diretamente no fluxo como um bloco de bytes contíguo.
#[inline]
fn write_slice<T: Copy, W: Write>(writer: &mut W, slice: &[T]) -> io::Result<()> {
    let bytes = unsafe {
        std::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            std::mem::size_of_val(slice),
        )
    };
    writer.write_all(bytes)
}

/// Lê bytes do fluxo diretamente para uma fatia contígua pré-alocada de dados.
#[inline]
fn read_slice<T: Copy, R: Read>(reader: &mut R, slice: &mut [T]) -> io::Result<()> {
    let bytes = unsafe {
        std::slice::from_raw_parts_mut(
            slice.as_mut_ptr() as *mut u8,
            std::mem::size_of_val(slice),
        )
    };
    reader.read_exact(bytes)
}

/// Grafo Esparso Comprimido (*Compressed Sparse Row* - CSR) em arquitetura SoA (*Structure of Arrays*).
///
/// Adota o padrão SoA, separando os atributos das arestas em dois vetores contíguos paralelos
/// (`arestas_destino` e `arestas_distancia`), eliminando 100% de bytes de preenchimento (*padding*),
/// maximizando a localidade espacial de cache L1/L2 e permitindo serialização/desserialização direta de blocos.
#[derive(Debug)]
pub struct Grafo {
    /// Deslocamentos acumulados (*offsets*) para indexar os vizinhos de cada nó nos vetores CSR.
    ///
    /// Os vizinhos do nó `u` estão situados no intervalo de fatia:
    /// `offsets[u] as usize .. offsets[u + 1] as usize`.
    /// O array possui tamanho fixo de $N + 1$ elementos.
    pub offsets: Box<[u32]>,

    /// Vetor contíguo CSR contendo apenas os identificadores de destino dos vizinhos (u16 puro, sem padding).
    pub arestas_destino: Box<[NodeId]>,

    /// Vetor contíguo CSR contendo apenas as distâncias dos trechos viários (f64 puro, sem padding).
    pub arestas_distancia: Box<[f64]>,

    /// Coordenadas métricas planas de cada nó, indexadas diretamente pelo seu [`NodeId`].
    pub coordenadas: Box<[Coordenada]>,

    /// Mapeamento do identificador interno compacto ([`NodeId`]) para o `osmid` original do OpenStreetMap.
    pub id_para_osmid: Box<[u64]>,

    /// Tabela hash para conversão rápida de `osmid` externo para o índice interno compacto [`NodeId`].
    pub osmid_para_id: HashMap<u64, NodeId>,
}

impl Grafo {
    /// Converte um identificador externo `osmid` para o identificador numérico interno [`NodeId`].
    ///
    /// # Retorno
    ///
    /// Retorna `Some(NodeId)` se o nó pertencer à malha viária, ou `None` caso contrário.
    #[inline(always)]
    pub fn osmid_para_id(&self, osmid: u64) -> Option<NodeId> {
        self.osmid_para_id.get(&osmid).copied()
    }

    /// Converte um [`NodeId`] interno de volta para o identificador global `osmid` correspondente.
    ///
    /// # Parâmetros
    ///
    /// * `id` - Identificador interno no intervalo `0..total_nos()-1`.
    #[inline(always)]
    pub fn id_para_osmid(&self, id: NodeId) -> u64 {
        self.id_para_osmid[id as usize]
    }

    /// Obtém a coordenada métrica plana associada a um nó da malha viária.
    ///
    /// # Parâmetros
    ///
    /// * `id` - Identificador interno do nó desejado.
    #[inline(always)]
    pub fn coordenada(&self, id: NodeId) -> Coordenada {
        self.coordenadas[id as usize]
    }

    /// Retorna a fatia de nós de destino vizinhos do nó fornecido.
    ///
    /// Permite percorrer conexões com máxima eficiência de cache sem carregar distâncias desnecessariamente.
    #[inline(always)]
    pub fn vizinhos_destinos(&self, id: NodeId) -> &[NodeId] {
        let inicio = self.offsets[id as usize] as usize;
        let fim = self.offsets[(id + 1) as usize] as usize;
        &self.arestas_destino[inicio..fim]
    }

    /// Retorna a fatia de distâncias das arestas que partem do nó fornecido.
    #[inline(always)]
    pub fn vizinhos_distancias(&self, id: NodeId) -> &[f64] {
        let inicio = self.offsets[id as usize] as usize;
        let fim = self.offsets[(id + 1) as usize] as usize;
        &self.arestas_distancia[inicio..fim]
    }

    /// Retorna um iterador sobre os vizinhos do nó fornecido na forma de [`Aresta`].
    #[inline(always)]
    pub fn vizinhos(&self, id: NodeId) -> impl Iterator<Item = Aresta> + '_ {
        let inicio = self.offsets[id as usize] as usize;
        let fim = self.offsets[(id + 1) as usize] as usize;
        self.arestas_destino[inicio..fim]
            .iter()
            .zip(&self.arestas_distancia[inicio..fim])
            .map(|(&destino, &distancia)| Aresta::new(destino, distancia))
    }

    /// Retorna a quantidade total de nós (cruzamentos) presentes no grafo.
    #[inline(always)]
    pub fn total_nos(&self) -> usize {
        self.coordenadas.len()
    }

    /// Retorna a quantidade total de conexões direcionadas ativas no grafo.
    #[inline(always)]
    pub fn total_arestas(&self) -> usize {
        self.arestas_destino.len()
    }

    /// Carrega o grafo a partir de um arquivo binário serializado pré-compilado (`rp.graph`).
    ///
    /// Realiza a validação do cabeçalho de 30 bytes, obtém as quantidades exatas de nós e arestas,
    /// aloca os blocos contíguos de memória (`Box<[T]>`) sem redundância e desserializa os dados
    /// em tempo constante de I/O em blocos inteiros, sem loops campo a campo.
    ///
    /// # Estrutura do Cabeçalho Binário (30 bytes)
    ///
    /// | Offset | Campo | Tipo | Descrição |
    /// |---|---|---|---|
    /// | `0..4` | Magic Number | `[u8; 4]` | Assinatura ASCII `"RPGR"` |
    /// | `4` | Versão | `u8` | Versão do formato (`2`) |
    /// | `5..13` | Tamanho Metadados | `usize` (8B) | Comprimento do bloco de metadados em bytes |
    /// | `13..21` | Quantidade de Nós | `usize` (8B) | Total de vértices na malha viária |
    /// | `21..29` | Quantidade de Arestas | `usize` (8B) | Total de arestas direcionadas CSR |
    /// | `29` | Sentinela | `u8` | Marcador de validação `0xFF` |
    ///
    /// # Parâmetros
    ///
    /// * `caminho` - Caminho para o arquivo binário em disco (ex: `"rp.graph"`).
    ///
    /// # Errors
    ///
    /// Retorna [`std::io::Error`] se:
    /// * O arquivo não puder ser aberto ou lido ([`io::ErrorKind::NotFound`], [`io::ErrorKind::PermissionDenied`]).
    /// * A assinatura do arquivo (*Magic Number*) for diferente de `"RPGR"` ([`io::ErrorKind::InvalidData`]).
    /// * A versão do arquivo for diferente de [`FORMAT_VERSION`] ([`io::ErrorKind::InvalidData`]).
    /// * O tamanho informado dos metadados for incompatível ([`io::ErrorKind::InvalidData`]).
    /// * O byte sentinela final do cabeçalho não for `0xFF` ([`io::ErrorKind::InvalidData`]).
    /// * Ocorrer corrupção de dados ou fim inesperado do arquivo durante a leitura ([`io::ErrorKind::UnexpectedEof`]).
    pub fn carregar_binario<P: AsRef<Path>>(caminho: P) -> io::Result<Self> {
        let file = File::open(caminho)?;
        let mut reader = BufReader::new(file);

        // 1. Validar Magic Number
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != MAGIC_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Assinatura do arquivo inválida: esperado {:?}, obtido {:?}",
                    MAGIC_BYTES, magic
                ),
            ));
        }

        // 2. Validar Versão
        let mut version = [0u8; 1];
        reader.read_exact(&mut version)?;
        if version[0] != FORMAT_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Versão do formato incompatível: esperado {}, obtido {}",
                    FORMAT_VERSION, version[0]
                ),
            ));
        }

        // 3. Ler usize de metadados (tamanho em bytes do bloco de metadados)
        let mut usize_buf = [0u8; std::mem::size_of::<usize>()];
        reader.read_exact(&mut usize_buf)?;
        let tam_metadados = usize::from_le_bytes(usize_buf);

        let tam_esperado: usize = std::mem::size_of::<usize>() * 2 + 1;
        if tam_metadados != tam_esperado {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Tamanho de metadados inválido: esperado {} bytes, obtido {} bytes",
                    tam_esperado, tam_metadados
                ),
            ));
        }

        // 4. Ler tamanhos de nós e arestas
        reader.read_exact(&mut usize_buf)?;
        let num_nos = usize::from_le_bytes(usize_buf);

        reader.read_exact(&mut usize_buf)?;
        let num_arestas = usize::from_le_bytes(usize_buf);

        // 5. Validar o byte sentinela de fim do cabeçalho
        let mut marker = [0u8; 1];
        reader.read_exact(&mut marker)?;
        if marker[0] != HEADER_MARKER {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Marcador de cabeçalho inválido: esperado 0x{:02X}, obtido 0x{:02X}",
                    HEADER_MARKER, marker[0]
                ),
            ));
        }

        // 6. Desserializar nós em blocos contíguos de memória
        let mut id_para_osmid = vec![0u64; num_nos];
        read_slice(&mut reader, &mut id_para_osmid)?;

        let mut coordenadas = vec![Coordenada::new(0.0, 0.0); num_nos];
        read_slice(&mut reader, &mut coordenadas)?;

        // Reconstruir o mapa osmid -> NodeId diretamente na heap
        let mut osmid_para_id = HashMap::with_capacity(num_nos);
        for (i, &osmid) in id_para_osmid.iter().enumerate() {
            osmid_para_id.insert(osmid, i as NodeId);
        }

        // 7. Desserializar offsets CSR em bloco contíguo
        let mut offsets = vec![0u32; num_nos + 1];
        read_slice(&mut reader, &mut offsets)?;

        // 8. Desserializar arestas SoA CSR em blocos contíguos (zero padding)
        let mut arestas_destino = vec![0 as NodeId; num_arestas];
        read_slice(&mut reader, &mut arestas_destino)?;

        let mut arestas_distancia = vec![0.0f64; num_arestas];
        read_slice(&mut reader, &mut arestas_distancia)?;

        Ok(Self {
            offsets: offsets.into_boxed_slice(),
            arestas_destino: arestas_destino.into_boxed_slice(),
            arestas_distancia: arestas_distancia.into_boxed_slice(),
            coordenadas: coordenadas.into_boxed_slice(),
            id_para_osmid: id_para_osmid.into_boxed_slice(),
            osmid_para_id,
        })
    }

    /// Serializa e grava a instância atual do grafo em disco no formato binário comprimido `rp.graph`.
    ///
    /// # Parâmetros
    ///
    /// * `caminho` - Destino em disco onde o arquivo binário será gravado.
    ///
    /// # Errors
    ///
    /// Retorna [`std::io::Error`] se ocorrer falha na criação do arquivo ou na escrita dos bytes.
    pub fn salvar_binario<P: AsRef<Path>>(&self, caminho: P) -> io::Result<()> {
        let file = File::create(caminho)?;
        let mut writer = BufWriter::new(file);

        let num_nos = self.coordenadas.len();
        let num_arestas = self.arestas_destino.len();

        // 1. Gravar cabeçalho: Magic Number e Versão
        writer.write_all(MAGIC_BYTES)?;
        writer.write_all(&[FORMAT_VERSION])?;

        // 2. usize de metadados: tamanho em bytes do bloco de metadados (num_nos + num_arestas + marcador)
        let tam_metadados: usize = std::mem::size_of::<usize>() * 2 + 1;
        writer.write_all(&tam_metadados.to_le_bytes())?;

        // 3. Metadados
        writer.write_all(&num_nos.to_le_bytes())?;
        writer.write_all(&num_arestas.to_le_bytes())?;
        writer.write_all(&[HEADER_MARKER])?;

        // 4. Gravar nós em blocos contíguos (SoA/Flat)
        write_slice(&mut writer, &self.id_para_osmid)?;
        write_slice(&mut writer, &self.coordenadas)?;

        // 5. Gravar offsets CSR em bloco contíguo
        write_slice(&mut writer, &self.offsets)?;

        // 6. Gravar arestas SoA CSR em blocos contíguos (destinos: u16 e distancias: f64)
        write_slice(&mut writer, &self.arestas_destino)?;
        write_slice(&mut writer, &self.arestas_distancia)?;

        writer.flush()?;
        Ok(())
    }

    /// Construtor alternativo: processa os arquivos CSV brutos da malha viária,
    /// constrói a estrutura CSR em memória e aplica deduplicação bidirecional.
    ///
    /// Lê as coordenadas e identificadores a partir do arquivo de nós e as ligações
    /// viárias a partir do arquivo de arestas. Como o grafo é não orientado, cada aresta
    /// é inserida nos dois sentidos ($u \to v$ e $v \to u$), mantendo a menor distância
    /// em caso de trechos paralelos duplicados.
    ///
    /// # Parâmetros
    ///
    /// * `caminho_nos` - Caminho para o CSV de nós contendo colunas `osmid`, `UTM_X_Este` e `UTM_Y_Norte`.
    /// * `caminho_arestas` - Caminho para o CSV de conexões contendo `origem`, `destino` e `distancia`.
    ///
    /// # Errors
    ///
    /// Retorna [`std::io::Error`] se algum arquivo CSV não for encontrado ou se colunas obrigatórias
    /// estiverem ausentes no cabeçalho.
    pub fn construir_dos_csvs<P1: AsRef<Path>, P2: AsRef<Path>>(
        caminho_nos: P1,
        caminho_arestas: P2,
    ) -> io::Result<Self> {
        // --- 1. Leitura dos Nós ---
        let file_nos = File::open(caminho_nos)?;
        let reader_nos = BufReader::new(file_nos);

        let mut id_para_osmid = Vec::new();
        let mut coordenadas = Vec::new();
        let mut osmid_para_id = HashMap::new();

        let mut lines_nos = reader_nos.lines();
        if let Some(header_line) = lines_nos.next() {
            let header = header_line?;
            let colunas: Vec<&str> = header.split(',').map(|s| s.trim()).collect();

            let osmid_idx = colunas
                .iter()
                .position(|&c| c.eq_ignore_ascii_case("osmid"))
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Coluna 'osmid' não encontrada"))?;
            let x_idx = colunas
                .iter()
                .position(|&c| c.eq_ignore_ascii_case("utm_x_este"))
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Coluna 'UTM_X_Este' não encontrada"))?;
            let y_idx = colunas
                .iter()
                .position(|&c| c.eq_ignore_ascii_case("utm_y_norte"))
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Coluna 'UTM_Y_Norte' não encontrada"))?;

            for line_res in lines_nos {
                let line = line_res?;
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let partes: Vec<&str> = trimmed.split(',').collect();
                if partes.len() <= x_idx.max(y_idx).max(osmid_idx) {
                    continue;
                }

                if let (Ok(osmid), Ok(x), Ok(y)) = (
                    partes[osmid_idx].trim().parse::<u64>(),
                    partes[x_idx].trim().parse::<f64>(),
                    partes[y_idx].trim().parse::<f64>(),
                ) {
                    let id = id_para_osmid.len() as NodeId;
                    id_para_osmid.push(osmid);
                    coordenadas.push(Coordenada::new(x, y));
                    osmid_para_id.insert(osmid, id);
                }
            }
        }

        let num_nos = id_para_osmid.len();
        let mut adj: Vec<HashMap<NodeId, f64>> = vec![HashMap::new(); num_nos];

        // --- 2. Leitura das Arestas ---
        let file_arestas = File::open(caminho_arestas)?;
        let reader_arestas = BufReader::new(file_arestas);
        let mut lines_arestas = reader_arestas.lines();

        if let Some(header_line) = lines_arestas.next() {
            let header = header_line?;
            let colunas: Vec<&str> = header.split(',').map(|s| s.trim()).collect();

            let origem_idx = colunas
                .iter()
                .position(|&c| c.eq_ignore_ascii_case("origem"))
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Coluna 'origem' não encontrada"))?;
            let destino_idx = colunas
                .iter()
                .position(|&c| c.eq_ignore_ascii_case("destino"))
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Coluna 'destino' não encontrada"))?;
            let dist_idx = colunas
                .iter()
                .position(|&c| c.eq_ignore_ascii_case("distancia"))
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Coluna 'distancia' não encontrada"))?;

            for line_res in lines_arestas {
                let line = line_res?;
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let partes: Vec<&str> = trimmed.split(',').collect();
                if partes.len() <= origem_idx.max(destino_idx).max(dist_idx) {
                    continue;
                }

                if let (Ok(origem_osm), Ok(destino_osm), Ok(distancia)) = (
                    partes[origem_idx].trim().parse::<u64>(),
                    partes[destino_idx].trim().parse::<u64>(),
                    partes[dist_idx].trim().parse::<f64>(),
                ) {
                    if let (Some(&u), Some(&v)) =
                        (osmid_para_id.get(&origem_osm), osmid_para_id.get(&destino_osm))
                    {
                        if u != v {
                            adj[u as usize]
                                .entry(v)
                                .and_modify(|d| *d = d.min(distancia))
                                .or_insert(distancia);
                            adj[v as usize]
                                .entry(u)
                                .and_modify(|d| *d = d.min(distancia))
                                .or_insert(distancia);
                        }
                    }
                }
            }
        }

        // --- 3. Construção do CSR SoA ---
        let mut offsets = Vec::with_capacity(num_nos + 1);
        let total_arestas: usize = adj.iter().map(|viz| viz.len()).sum();
        let mut arestas_destino = Vec::with_capacity(total_arestas);
        let mut arestas_distancia = Vec::with_capacity(total_arestas);

        let mut acumulado: u32 = 0;
        for vizinhos in &adj {
            offsets.push(acumulado);
            for (&viz_id, &dist) in vizinhos {
                arestas_destino.push(viz_id);
                arestas_distancia.push(dist);
            }
            acumulado = arestas_destino.len() as u32;
        }
        offsets.push(acumulado);

        Ok(Self {
            offsets: offsets.into_boxed_slice(),
            arestas_destino: arestas_destino.into_boxed_slice(),
            arestas_distancia: arestas_distancia.into_boxed_slice(),
            coordenadas: coordenadas.into_boxed_slice(),
            id_para_osmid: id_para_osmid.into_boxed_slice(),
            osmid_para_id,
        })
    }

    /// Ponto de entrada recomendado para carregamento do grafo.
    ///
    /// Tenta carregar diretamente o binário `rp.graph`. Caso o arquivo não exista ou esteja em versão
    /// incompatível com a versão atual, executa automaticamente a construção a partir dos CSVs da pasta `data/`,
    /// salva o arquivo `rp.graph` para persistência e retorna o grafo pronto.
    ///
    /// # Errors
    ///
    /// Retorna [`std::io::Error`] se ocorrer falha na leitura dos dados ou na criação do binário.
    pub fn carregar() -> io::Result<Self> {
        let bin_path = "rp.graph";
        if Path::new(bin_path).exists() {
            match Self::carregar_binario(bin_path) {
                Ok(grafo) => Ok(grafo),
                Err(e) => {
                    eprintln!(
                        "Aviso: arquivo rp.graph incompatível ou desatualizado ({}). Reconstruindo a partir dos CSVs...",
                        e
                    );
                    let grafo = Self::construir_dos_csvs("data/coordenadas_finais.csv", "data/arestas.csv")?;
                    grafo.salvar_binario(bin_path)?;
                    Ok(grafo)
                }
            }
        } else {
            let grafo = Self::construir_dos_csvs("data/coordenadas_finais.csv", "data/arestas.csv")?;
            grafo.salvar_binario(bin_path)?;
            Ok(grafo)
        }
    }
}
