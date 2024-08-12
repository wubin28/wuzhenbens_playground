import random

# 基本句子列表
sentences = [
    "The quick brown fox jumps over the lazy dog.",
    "All work and no play makes Jack a dull boy.",
    "To be or not to be, that is the question.",
    "I think, therefore I am.",
    "Life is like a box of chocolates.",
    "May the Force be with you.",
    "Elementary, my dear Watson.",
    "Houston, we have a problem.",
    "E.T. phone home.",
    "There's no place like home.",
]

# 额外单词列表，用于增加变化
extra_words = [
    "computer", "algorithm", "programming", "python", "concurrency",
    "multithreading", "performance", "optimization", "analysis", "design",
    "software", "hardware", "network", "database", "artificial",
    "intelligence", "machine", "learning", "data", "science",
]

def generate_sentence():
    # 随机选择一个基本句子
    sentence = random.choice(sentences)

    # 有 30% 的机会在句子中添加额外的单词
    if random.random() < 0.3:
        words = sentence.split()
        insert_position = random.randint(0, len(words))
        words.insert(insert_position, random.choice(extra_words))
        sentence = " ".join(words)

    return sentence

# 生成大约 0.2 MB 的文本
target_size = 0.2 * 1024 * 1024  # 0.2 MB
current_size = 0
with open("input.txt", "w") as f:
    while current_size < target_size:
        sentence = generate_sentence()
        f.write(sentence + "\n")
        current_size += len(sentence) + 1  # +1 for newline character

print(f"Generated input.txt with size: {current_size / (1024 * 1024):.2f} MB")