// Charlie Tanfoll Pereira Lobo - 16827968
// Marina Cintra Queiroz - 17074404

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::graph::Grafo;
use crate::structs::{NodeId, ResultadoBusca};

/// Elemento armazenado na fronteira de busca do A* (*A-Star*).
///
/// Contém o identificador compacto do nó, a função de prioridade total $f(n)$ e a
/// estimativa heurística $h(n)$ utilizada como critério de desempate.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NoAStar {
    /// Identificador compacto do nó no grafo.
    pub id: NodeId,
    /// Custo de avaliação total: $f(n) = w_g \cdot g(n) + w_h \cdot h(n)$.
    pub f: f64,
    /// Estimativa heurística euclidiana $h(n)$ até o destino (critério de desempate).
    pub h: f64,
}

impl Eq for NoAStar {}

impl Ord for NoAStar {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        // Implementação de Min-Heap:
        // O nó com MENOR valor de f(n) ganha a maior prioridade.
        // Em caso de empate no custo f(n), prioriza o nó com MENOR heurística h(n)
        // (isto é, o nó fisicamente mais próximo do destino em linha reta).
        // `total_cmp` lida de forma estrita e segura com números de ponto flutuante (f64).
        match other.f.total_cmp(&self.f) {
            Ordering::Equal => other.h.total_cmp(&self.h),
            ordem => ordem,
        }
    }
}

impl PartialOrd for NoAStar {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Min-Heap especializado para a fronteira do algoritmo A* (*A-Star*).
///
/// Encapsula a fila de prioridade garantindo inserção em $O(\log N)$ e extração
/// do nó com menor custo total estimado em $O(\log N)$.
#[derive(Debug, Default)]
pub struct MinHeapAStar {
    heap: BinaryHeap<NoAStar>,
}

impl MinHeapAStar {
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

    /// Insere um nó na fronteira com seu custo avaliado $f(n)$ e heurística $h(n)$.
    #[inline]
    pub fn push(&mut self, id: NodeId, f: f64, h: f64) {
        self.heap.push(NoAStar { id, f, h });
    }

    /// Remove e retorna o nó com menor custo $f(n)$ da fronteira.
    #[inline]
    pub fn pop(&mut self) -> Option<NoAStar> {
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
    /// Executa o algoritmo **A* (A-Star)** padrão entre dois nós identificados por `osmid`.
    ///
    /// Por padrão, utiliza pesos balanceados iguais ($w_g = 0.5$ e $w_h = 0.5$),
    /// conforme os requisitos do trabalho para o Desafio 1 (Rota Ótima).
    ///
    /// # Retorno
    ///
    /// Retorna `Some(ResultadoBusca)` se um caminho for encontrado, ou `None` caso contrário.
    #[inline]
    pub fn a_star(&self, origem_osmid: u64, destino_osmid: u64) -> Option<ResultadoBusca> {
        self.a_star_ponderado(origem_osmid, destino_osmid, 0.5, 0.5)
    }

    /// Executa a busca **A* Ponderado (*Weighted A\*)** permitindo ajustar os pesos do custo do caminho e da heurística.
    ///
    /// A função de avaliação é calculada como:
    /// $$f(n) = \text{peso\_caminho} \cdot g(n) + \text{peso\_heuristica} \cdot h(n)$$
    ///
    /// # Parâmetros
    ///
    /// * `origem_osmid` - OSM ID do cruzamento de partida.
    /// * `destino_osmid` - OSM ID do cruzamento de destino final.
    /// * `peso_caminho` - Peso $w_g$ atribuído à distância real acumulada $g(n)$.
    /// * `peso_heuristica` - Peso $w_h$ atribuído à estimativa euclidiana $h(n)$.
    ///
    /// # Retorno
    ///
    /// Retorna `Some(ResultadoBusca)` contendo a rota, distância e nós expandidos, ou `None` se inalcançável.
    pub fn a_star_ponderado(
        &self,
        origem_osmid: u64,
        destino_osmid: u64,
        peso_caminho: f64,
        peso_heuristica: f64,
    ) -> Option<ResultadoBusca> {
        let id_origem = self.osmid_para_id(origem_osmid)?;
        let id_destino = self.osmid_para_id(destino_osmid)?;

        let total_nos = self.total_nos();
        let coordenada_destino = self.coordenada(id_destino);

        let mut fronteira = MinHeapAStar::new();
        let mut visitados = vec![false; total_nos];
        let mut custos_g = vec![f64::INFINITY; total_nos];
        let mut veio_de: Vec<Option<NodeId>> = vec![None; total_nos];
        let mut nos_expandidos: usize = 0;

        // Configuração inicial do nó de origem
        custos_g[id_origem as usize] = 0.0;
        let heuristica_origem = self.coordenada(id_origem).distancia_euclidiana(&coordenada_destino);
        let prioridade_origem_f = peso_caminho * 0.0 + peso_heuristica * heuristica_origem;
        fronteira.push(id_origem, prioridade_origem_f, heuristica_origem);

        while let Some(no_fronteira) = fronteira.pop() {
            let id_no_atual = no_fronteira.id;

            // Se o nó já foi expandido anteriormente com um caminho menor ou igual, ignora
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

            let custo_g_atual = custos_g[id_no_atual as usize];

            // Gera e avalia os vizinhos para inserção na fronteira
            for vizinho in self.vizinhos(id_no_atual) {
                let id_vizinho = vizinho.destino;

                if visitados[id_vizinho as usize] {
                    continue;
                }

                let novo_custo_g = custo_g_atual + vizinho.distancia;

                if novo_custo_g < custos_g[id_vizinho as usize] {
                    custos_g[id_vizinho as usize] = novo_custo_g;
                    veio_de[id_vizinho as usize] = Some(id_no_atual);

                    let heuristica_vizinho = self.coordenada(id_vizinho).distancia_euclidiana(&coordenada_destino);
                    let prioridade_vizinho_f = peso_caminho * novo_custo_g + peso_heuristica * heuristica_vizinho;
                    fronteira.push(id_vizinho, prioridade_vizinho_f, heuristica_vizinho);
                }
            }
        }

        None
    }
}