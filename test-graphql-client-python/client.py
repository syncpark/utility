from python_graphql_client import GraphqlClient
import urllib3


def login(target, id, password):
    client = GraphqlClient(
        endpoint=target, verify=False)
    login = """
        mutation login($id:String, $password: String) {
            signIn(username:$id, password:$password) {
                token
            }
        }
    """
    login_variable = {'id': id, 'password': password}
    data = client.execute(query=login, variables=login_variable)
    token = data['data']['signIn']['token']
    return token


def get_sources(target, token):
    header = {
        'Authorization': 'Bearer ' + token
    }

    client = GraphqlClient(
        endpoint=target, headers=header, verify=False)

    query = """
        query sources {
            sources
        }
    """
    sources = client.execute(query=query)
    node_list = sources['data']['sources']
    return node_list


def parse_and_save_log(edges, path):
    writer = open(path, "+a")
    start = ''
    for node in edges:
        start = node['node']['timestamp']
        respPort = node['node']['respPort']
        host = node['node']['host']
        uri = node['node']['uri']
        if host == '-' and uri == '-':
            continue
        if not host:
            host = "-"
        if respPort == 443:
            writer.write('https://' + host + '\n')
        else:
            writer.write('http://' + host + uri + '\n')
    return start


def process_log(target, token, source, start, end):
    header = {
        'Authorization': 'Bearer ' + token
    }

    client = GraphqlClient(
        endpoint=target, headers=header, verify=False)

    while start:
        http_query = """
            query http($source: String, $start: String, $end: String) {
            httpRawEvents (filter: {source:$source, time: {start: $start, end: $end}}, first: 100) {
                edges {
                node {
                    timestamp
                    respPort
                    host
                    uri
                }
                }
            }
            }
        """
        http_variables = {"source": source}

        data = client.execute(query=http_query, variables=http_variables)
        edges = data['data']['httpRawEvents']['edges']
        start = parse_and_save_log(edges, "test.log")


urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

token = login("https://172.30.1.216:8443/graphql", "admin", "admin")
sources = get_sources("https://172.30.1.216:8443/archive", token)
for source in sources:
    process_log("https://172.30.1.216:8443/archive", token,
                source, "2023-04-26T15:00:00.000000000+00:00", "2023-04-26T19:00:00.000000000+00:00")
