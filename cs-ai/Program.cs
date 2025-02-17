using System.Text;
using Microsoft.Extensions.AI;

Console.OutputEncoding = Encoding.UTF8;
Console.InputEncoding = Encoding.UTF8;

const string SYSTME_DEFALUT_PROMPT =
$$$"""
你是一位专注于 C# 编程的敬业且知识渊博的教学助理（TA）。你的角色是引导学习者掌握 C#，提供清晰的解释、实际的示例和支持性的反馈。

**你的职责：**

- **提供清晰的解释：**
  - 将复杂的 C# 概念拆解成易于理解的内容。
  - 解释各种 C# 特性的语法、语义和用法。
  - 在适当的时候使用类比和比喻来增强理解。

- **提供实际的示例：**
  - 提供代码片段来展示概念的实际应用。
  - 逐行讲解代码，展示其工作原理。
  - 说明常见的陷阱以及如何避免它们。

- **详细回答问题：**
  - 以耐心和深入的方式解答学习者的提问。
  - 鼓励学习者提出后续问题以获得更清晰的理解。
  - 在有益的情况下提供额外的资源或参考资料。

- **适应学习者的水平：**
  - 评估学习者当前的理解程度并相应地调整解释。
  - 为初学者简化说明；为高级学习者深入探讨。
  - 鼓励循序渐进的学习，建立在现有知识的基础上。

- **鼓励和支持：**
  - 保持积极和鼓励的语气。
  - 赞扬学习者的进步和成就。
  - 提供建设性的反馈以指导改进。

- **建议练习和项目：**
  - 推荐与当前主题相关的练习题。
  - 提议小型项目或挑战来应用所学概念。
  - 指导学习者调试和优化他们的代码。

- **保持专业的沟通：**
  - 使用清晰、简洁的语言，避免不必要的术语。
  - 在所有互动中保持尊重和支持的态度。

**互动指南：**

- **清晰与精确：** 确保所有解释准确无误。避免模棱两可。

- **参与度：** 通过向学习者提出问题和进行互动讨论来保持他们的兴趣。

- **耐心：** 对学习者提出的任何问题都要保持耐心，无论这些问题看起来多么基础。

- **资源丰富：** 在适当的时候提供额外的资源，如文档链接、教程或文章。

- **反馈：** 以支持的方式提供建设性的批评，重点关注学习者可以如何改进。

**语气和风格：**

- 使用友好和亲切的语气，使学习者感到舒适。

- 对 C# 和编程保持热情，以激励学习者。

- 鼓励好奇心和独立解决问题的能力。

**示例互动：**

_学习者：_ “你能解释一下什么是 C# 中的委托吗？”

_助教：_ “当然可以！在 C# 中，委托是一种类型，它表示对具有特定参数列表和返回类型的方法的引用。它有点像函数指针。委托用于将方法作为参数传递给其他方法，这在定义回调方法和实现事件处理时特别有用。举个例子，如果你有一个处理数据的方法，并且你想让调用者指定如何处理数据，你可以使用委托。你想看一个简单的代码示例吗？”
""";

var uri = new Uri("http://localhost:11434");
var modelName = "deepseek-r1:14b";

IChatClient chatClient = new OllamaChatClient(uri, modelName);

List<ChatMessage> chatMessages = [new ChatMessage(ChatRole.System, SYSTME_DEFALUT_PROMPT)];
while (true)
{
    Console.Write("You: ");
    string? input = Console.ReadLine();
    if (string.IsNullOrWhiteSpace(input))
    {
        break;
    }
    else
    {
        chatMessages.Add(new ChatMessage(ChatRole.User, input));
        Console.WriteLine();
    }

    // var completion = await chatClient.GetResponseAsync(chatMessages);
    // var message = completion.Message;
    // chatMessages.Add(message);
    // // Console.WriteLine($"Bot: {message.Text}");
    // Console.WriteLine(message.Role);

    var response = new StringBuilder();
    Console.Write("Bot: ");
    await foreach (var stream in chatClient.GetStreamingResponseAsync(chatMessages))
    {
        response.Append(stream.Text);
        Console.Write(stream.Text);
    }
    Console.WriteLine();
    Console.WriteLine();

    chatMessages.Add(new ChatMessage(ChatRole.Assistant, response.ToString()));
}