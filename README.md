# WBTech Task 00 - Демонстрационный сервис, отображающий данные о заказе

## Описание
Небольшой веб-сервис для отображения данных о заказе.

## Нюансы
Я делал все второпях, поэтому в коде есть пара вопросов и проблем:
1. Не очень красиво сделал модели сериализации и десереализации.
2. ОЧЕНЬ ужасная архитектура БД. Я согласен, что она непрактичная и т.д., но давайте поймем и простим.
3. По-хорошему все сервисы/views стоит перенести в другой файл.
4. Кэш не обладает таймаутом.

## Запуск
1. Для начала стоит запустить докер-контейнер с БД:
```bash
docker-compose up -d
```
2. После этого можно запустить крейт:
```bash
cargo run
```
3. Готово.

## Использование/Тестирование
Я не делал никаких тестов, но можно использовать curl или Postman:
1. Получение списка заказов:
```bash
curl -X GET http://localhost:8080/orders
```
2. Получение информации о заказе:
```bash
curl -X GET http://localhost:8080/orders/1
```
3. Создание заказа:
```bash
curl -X POST http://localhost:8080/orders -H "Content-Type: application/json" -d '{
  "order_uid": "2",
  "track_number": "DDSA2138",
  "entry": "Entry1223",
  "delivery": {
    "name": "John Doe",
    "phone": "1234567890",
    "zip": "12345",
    "city": "CityName",
    "address": "123 Street",
    "region": "RegionName",
    "email": "john.doe@example.com"
  },
  "payment": {
    "transaction": "TX12345",
    "request_id": "REQ12345",
    "currency": "USD",
    "provider": "ProviderName",
    "amount": 100,
    "payment_dt": 1234567890,
    "bank": "BankName",
    "delivery_cost": 10,
    "goods_total": 90,
    "custom_fee": 5
  },
  "items": [
    {
      "chrt_id": 1,
      "track_number": "TN12345",
      "price": 50,
      "rid": "RID123",
      "name": "ItemName",
      "sale": 5,
      "size": "L",
      "total_price": 45,
      "nm_id": 1,
      "brand": "BrandName",
      "status": 0
    }
  ],
  "locale": "en",
  "internal_signature": "sig123",
  "customer_id": "cust123",
  "delivery_service": "DHL",
  "shared_key": "sharedKey123",
  "sm_id": 1,
  "date_created": "2024-09-27",
  "oof_shard": "shard1"
}'
```
4. Обновление заказа:
```bash
curl -X PUT http://localhost:8080/orders/1 -H "Content-Type: application/json" -d '{
    "track_number": "NewTrack_number8",
    "entry": "NewEntry8",
}'
```
При обновлении заказа можно обновить любые поля, которые необходимо обновить, кроме `order_uid`, он подгружается автоматически.