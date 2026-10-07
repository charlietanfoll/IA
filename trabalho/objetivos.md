# Objetivos do Trabalho de Inteligência Artificial

O objetivo principal deste trabalho é **desenvolver um sistema de roteamento capaz de encontrar caminhos válidos em uma malha viária real**. 

Para isso, serão utilizados os dados da cidade de Ribeirão Preto extraídos do OpenStreetMap (OSM), simplificados em dois arquivos tabulares (`nos.csv` e `arestas.csv`) que representam o espaço de estados (um grafo não orientado). O sistema deve receber um nó de origem e um nó de destino e calcular a rota entre eles.

O trabalho é dividido em dois desafios principais com objetivos específicos:

## Desafio 1: A Rota Ótima
- **Objetivo:** Encontrar, obrigatoriamente, a rota com a **distância mínima** (o caminho ótimo) entre a origem e o destino.
- **Avaliação (Filtro da Otimidade):** Se o caminho encontrado for mais longo que o ótimo, o algoritmo falha.
- **Desempate:** Em caso de empate na distância, o desempate é feito pela menor quantidade de **nós expandidos** (eficiência).

## Desafio 2: A Rota Expressa Subótima
- **Objetivo:** Priorizar a **velocidade extrema**. O sistema pode abrir mão da precisão absoluta para expandir o menor número de nós possível.
- **Avaliação (Filtro de Tolerância):** O caminho retornado não precisa ser perfeito, mas a distância total deve ser, no máximo, **15% maior** do que a distância ótima.
- **Desempate (Eficiência Extrema):** Entre as rotas válidas dentro da tolerância, vence o algoritmo que for mais veloz (menor número de nós expandidos).

## Outros Objetivos e Requisitos
- **Visualização:** Utilizar o script `verRota.py` fornecido para plotar e visualizar o caminho encontrado em um mapa interativo.
- **Métricas:** Medir e apresentar corretamente a Distância Total da Rota e a Contagem de Nós Expandidos (seguindo a definição formal de expansão estipulada no documento).
- **Implementação:** Garantir clareza, modularidade do código e uma documentação intracódigo bem feita. A saída deve ser impressa no terminal seguindo um formato (template) estrito.
- **Apresentação:** Preparar uma apresentação com slides para demonstrar a escolha dos algoritmos, estruturas de dados utilizadas, os desafios enfrentados, os resultados visuais e avaliar os *trade-offs* entre as abordagens (otimalidade vs. velocidade).
