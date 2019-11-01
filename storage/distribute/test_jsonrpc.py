import requests
import string
import random
import time

url = "http://127.0.0.1:3030"

def read(id: str):
    jsonrpc = {
        "jsonrpc": "2.0",
        "id": "0",
        "method": "local",
        "params": {
            "method": "read",
            "params": {
                "id": id
            }
        }
    }

    res = requests.post(url, json=jsonrpc)
    print(res.json())

def write(data: str):
    jsonrpc = {
        "jsonrpc": "2.0",
        "id": "0",
        "method": "local",
        "params": {
            "method": "write",
            "params": {
                "data": data
            }
        }
    }

    res = requests.post(url, json=jsonrpc)
    print(res.json())

def write_one():
    write(''.join(random.choice(string.ascii_lowercase)
                  for _ in range(20)))

def write_times(n):
    for i in range(n):
        write_one()

if __name__ == "__main__":
    #write_times(10)
    read("0x6d9b3ba7729d5e235194a52f8a5f9b734418599e2b1a1309d76c294d1f92cae6")
