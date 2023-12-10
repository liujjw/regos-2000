#include <stdio.h>
#include <stdlib.h>
#include "queue.h"

Queue* createQueue() {
    Queue* q = (Queue*)malloc(sizeof(Queue));
    q->front = q->rear = NULL;
    return q;
}

void enqueue(Queue* q, char* buf, int len, int start) {
    Node* temp = (Node*)malloc(sizeof(Node));
    temp->buf = buf;
    temp->len = len;
    temp->start = start;
    temp->next = NULL;
    if (q->rear == NULL) {
        q->front = q->rear = temp;
        return;
    }
    q->rear->next = temp;
    q->rear = temp;
}

ReturnData dequeue(Queue* q) {
    if (q->front == NULL)
        return -1;

    Node* temp = q->front;
    ReturnData value = ReturnData(temp->buf, temp->len, temp->start);

    q->front = q->front->next;
    if (q->front == NULL)
        q->rear = NULL;

    free(temp);

    return value;
}