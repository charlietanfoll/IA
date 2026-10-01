import pandas as pd
from pyproj import Transformer


def converter_coordenadas_csv(arquivo_entrada, arquivo_saida):
    """Lê um arquivo CSV com coordenadas Lat/Long de Ribeirão Preto

    e exporta um novo CSV com as colunas X (Este) e Y (Norte) em UTM (SIRGAS
    2000 / 22S).
    """
    print(f"Lendo o arquivo: {arquivo_entrada}...")
    # Carrega o CSV
    df = pd.read_csv(arquivo_entrada)

    # Verifica se as colunas necessárias existem (tratando variações de maiúsculas/minúsculas)
    colunas_obrigatorias = ["Osmid", "Latitude", "Longitude"]
    for col in colunas_obrigatorias:
        if col not in df.columns:
            raise ValueError(
                f"A coluna '{col}' não foi encontrada no arquivo CSV."
            )

    print("Calculando a conversão de alta precisão (EPSG:31982)...")
    # Configura o transformador oficial para a região de Ribeirão Preto
    # Entrada: SIRGAS 2000 Geográficas (EPSG:4674) -> Saída: UTM 22S (EPSG:31982)
    transformer = Transformer.from_crs("epsg:4674", "epsg:31982", always_xy=True)

    # Realiza a conversão vetorizada de alta performance
    # Importante: O pyproj sempre recebe a Longitude primeiro quando always_xy=True
    x, y = transformer.transform(
        df["Longitude"].values, df["Latitude"].values
    )

    # Adiciona as novas colunas ao DataFrame com precisão de 4 casas decimais (milimétrica)
    df["UTM_X_Este"] = [round(val, 4) for val in x]
    df["UTM_Y_Norte"] = [round(val, 4) for val in y]

    # Salva o resultado no novo arquivo CSV
    df.to_csv(arquivo_saida, index=False)
    print(f"Sucesso! Arquivo convertido salvo em: {arquivo_saida}")


# --- COMO EXECUTAR ---
# Substitua pelos nomes reais dos seus arquivos
arquivo_input = "nos.csv"
arquivo_output = "coordenadas_ribeirao_utm.csv"

# Executa a função
converter_coordenadas_csv(arquivo_input, arquivo_output)
