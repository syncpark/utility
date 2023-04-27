from python_graphql_client import GraphqlClient


client = GraphqlClient(
    endpoint="https://172.30.1.216:8443/graphql", verify=False)


login = """
    mutation {
        signIn(username:"admin", password:"admin") {
            token
        }
    }
"""
data = client.execute(query=login)
token = data['data']['signIn']['token']

header = {
    'Authorization': 'Bearer ' + token
}


client = GraphqlClient(
    endpoint="https://172.30.1.216:8443/archive", headers=header, verify=False)

query = """
    query sources {
        sources
    }
"""
sources = client.execute(query=query)
node_list = sources['data']['sources']
print(node_list)

http_query = """
    query http($source: String) {
    httpRawEvents (filter: {source:$source}, last: 10) {
        edges {
        node {
            timestamp
            origAddr
            origPort
            respAddr
            respPort
            method
            host
            uri
        }
        }
    }
    }
"""
http_variables = {"source": "collect"}

# Synchronous request
data = client.execute(query=http_query, variables=http_variables)

# data = {
#     'data': {
#         'httpRawEvents': {
#             'edges': [{
#                 'node': {
#                     'timestamp': '...',
#                     'origAddr': '...'
#                 }
#             }]
#         }
#     }
# }
edges = data['data']['httpRawEvents']['edges']

for node in edges:
    dstport = node['node']['respPort']
    method = node['node']['method']
    if not method:
        method = "-"
    host = node['node']['host']
    if not host:
        host = "-"
    uri = node['node']['uri']
    if not uri:
        uri = "-"
    if dstport != 443:
        print('http://' + host + uri)
