typedef struct Node {
  char* buf;
  int len;
  int start;
  struct Node* next;
} Node;

typedef struct ReturnData {
  char* buf;
  int len;
  int start;
} ReturnData;

typedef struct Queue {
  Node* front;
  Node* rear;
} Queue;

Queue* createQueue();
void enqueue(Queue* q, char* buf, int len, int start);
ReturnData dequeue(Queue* q);