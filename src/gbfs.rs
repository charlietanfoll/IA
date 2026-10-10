// Charlie Tanfoll Pereira Lobo - 16827968
// Marina Cintra Queiroz - 17074404

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::graph::Grafo;
use crate::structs::NodeId;

pub use crate::structs::ResultadoBusca;

/// Elemento armazenado na fronteira de busca do GBFS.
///
/// Contém o identificador do nó e sua estimativa heurística $h(n)$ até o destino.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NoGBFS {
    /// Identificador compacto do nó no grafo.
    pub id: NodeId,
    /// Distância euclidiana em linha reta (em metros) do nó até o destino final: $h(n)$.
    pub h: f64,
}

impl Eq for NoGBFS {}

impl Ord for NoGBFS {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        // Implementação de Min-Heap:
        // Por padrão o `BinaryHeap` do Rust é um Max-Heap (o maior elemento sai primeiro).
        // Ao inverter a comparação (`other.h` comparado a `self.h`), o nó com o MENOR valor
        // de h(n) ganha a maior prioridade e sairá primeiro do heap.
        // `total_cmp` lida de forma segura com números de ponto flutuante (f64).
        other.h.total_cmp(&self.h)
    }
}

impl PartialOrd for NoGBFS {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Min-Heap especializado para a fronteira do algoritmo GBFS (*Greedy Best-First Search*).
///
/// Encapsula a fila de prioridade garantindo inserção em $O(\log N)$ e extração
/// do nó com menor heurística $h(n)$ em $O(\log N)$.
#[derive(Debug, Default)]
pub struct MinHeapGBFS {
    heap: BinaryHeap<NoGBFS>,
}

impl MinHeapGBFS {
    /// Cria uma nova fronteira vazia.
    #[inline]
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    /// Cria uma nova fronteira pré-alocando capacidade inicial de nós.
    #[inline]
    pub fn with_capacity(capacidade: usize) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(capacidade),
        }
    }

    /// Insere um nó na fronteira com seu valor heurístico correspondente $h(n)$.
    #[inline]
    pub fn push(&mut self, id: NodeId, h: f64) {
        self.heap.push(NoGBFS { id, h });
    }

    /// Remove e retorna o nó com a menor distância heurística $h(n)$ da fronteira.
    #[inline]
    pub fn pop(&mut self) -> Option<NoGBFS> {
        self.heap.pop()
    }

    /// Retorna `true` se a fronteira não contiver nenhum nó pendente.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Retorna a quantidade de nós atualmente armazenados na fronteira.
    #[inline]
    pub fn len(&self) -> usize {
        self.heap.len()
    }
}

impl Grafo {
    /// Executa o algoritmo **Greedy Best-First Search (GBFS)** entre dois nós identificados por `osmid`.
    ///
    /// O algoritmo foca em alcançar o destino expandindo sempre o nó que possui a menor
    /// distância euclidiana em linha reta até o alvo ($f(n) = h(n)$).
    ///
    /// # Retorno
    ///
    /// Retorna `Some(ResultadoBusca)` se um caminho for encontrado, ou `None` caso contrário.
    pub fn gbfs(&self, origem_osmid: u64, destino_osmid: u64) -> Option<ResultadoBusca> {
        let id_origem = self.osmid_para_id(origem_osmid)?;
        let id_destino = self.osmid_para_id(destino_osmid)?;

        let total_nos = self.total_nos();
        let coordenada_destino = self.coordenada(id_destino);

        let mut fronteira = MinHeapGBFS::new();
        let mut visitados = vec![false; total_nos];
        let mut veio_de: Vec<Option<NodeId>> = vec![None; total_nos];
        let mut nos_expandidos: usize = 0;

        // Inserir nó de origem com sua heurística h(origem)
        let heuristica_origem = self.coordenada(id_origem).distancia_euclidiana(&coordenada_destino);
        fronteira.push(id_origem, heuristica_origem);

        while let Some(no_fronteira) = fronteira.pop() {
            let id_no_atual = no_fronteira.id;

            // Se o nó já foi expandido anteriormente, ignora
            if visitados[id_no_atual as usize] {
                continue;
            }

            // CRITÉRIO FORMAL DE EXPANSÃO (Especificação do Trabalho):
            // "Como g é o nó final, a busca termina... O contador não incrementa nesta etapa
            // porque o nó g foi apenas testado, não expandido."
            if id_no_atual == id_destino {
                // Reconstrução do caminho ótimo/encontrado
                let mut caminho_ids = Vec::new();
                let mut id_passo_atual = id_destino;
                caminho_ids.push(id_passo_atual);

                while let Some(id_predecessor) = veio_de[id_passo_atual as usize] {
                    caminho_ids.push(id_predecessor);
                    id_passo_atual = id_predecessor;
                }
                caminho_ids.reverse();

                // Calcular a distância total da rota somando as distâncias das arestas percorridas
                let mut distancia_total = 0.0;
                for trecho in caminho_ids.windows(2) {
                    let id_origem_trecho = trecho[0];
                    let id_destino_trecho = trecho[1];
                    let idx_inicio_arestas = self.offsets[id_origem_trecho as usize] as usize;
                    let idx_fim_arestas = self.offsets[(id_origem_trecho + 1) as usize] as usize;

                    for idx_aresta in idx_inicio_arestas..idx_fim_arestas {
                        if self.arestas_destino[idx_aresta] == id_destino_trecho {
                            distancia_total += self.arestas_distancia[idx_aresta];
                            break;
                        }
                    }
                }

                let caminho_osmid: Vec<u64> = caminho_ids
                    .iter()
                    .map(|&id_no| self.id_para_osmid(id_no))
                    .collect();

                return Some(ResultadoBusca {
                    caminho: caminho_osmid,
                    distancia_total,
                    nos_expandidos,
                });
            }

            // O nó não é o objetivo: marca como visitado e contabiliza a expansão
            visitados[id_no_atual as usize] = true;
            nos_expandidos += 1;

            // Gera e insere os vizinhos na fronteira
            for vizinho in self.vizinhos(id_no_atual) {
                let id_vizinho = vizinho.destino;
                if !visitados[id_vizinho as usize] && veio_de[id_vizinho as usize].is_none() && id_vizinho != id_origem {
                    veio_de[id_vizinho as usize] = Some(id_no_atual);
                    let heuristica_vizinho = self.coordenada(id_vizinho).distancia_euclidiana(&coordenada_destino);
                    fronteira.push(id_vizinho, heuristica_vizinho);
                }
            }
        }

        None
    }
}