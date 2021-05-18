import requests

url = ('https://newsapi.org/v2/top-headlines?'
       'country=us&'
       'apiKey=f11bda4e71fc4ec196ab969412b3cf0a')
print("Getting latest news")

response = requests.get(url)
text_file = open("LatestNews.txt","w",encoding="utf8")
eenlijn = ""

print("Making LatestNews.txt file")
for i in range(len(response.json()["articles"])):
       antwoord = response.json()["articles"][i]
       eenlijn = eenlijn + antwoord['title'] + " | "

text_file.write(eenlijn)
text_file.close()
print("File ready!")