import requests
import random
import string
import json
import concurrent.futures
import os

url = "http://localhost:8080"
payment_ids = []


def generate_random_id(prefix, length):
    characters = string.ascii_letters + string.digits
    random_id = "".join(random.choices(characters, k=length))

    return f"{prefix}_{random_id}"


def create_merchant_account():
    merchant_id = generate_random_id("mer", 20)
    acccount_url = f"{url}/accounts"

    payload = json.dumps({
        "merchant_id": merchant_id,
        "locker_id": "m0010",
        "merchant_name": "NewAge Retailer",
        "merchant_details": {
            "primary_contact_person": "John Test",
            "primary_email": "JohnTest@test.com",
            "primary_phone": "sunt laborum",
            "secondary_contact_person": "John Test2",
            "secondary_email": "JohnTest2@test.com",
            "secondary_phone": "cillum do dolor id",
            "website": "www.example.com",
            "about_business": "Online Retail with a wide selection of organic products for North America",
            "address": {
                "line1": "1467",
                "line2": "Harrison Street",
                "line3": "Harrison Street",
                "city": "San Fransico",
                "state": "California",
                "zip": "94122",
                "country": "US"
            }
        },
        "routing_algorithm": {
            "type": "single",
            "data": "stripe_test"
        },
        "return_url": "http://www.example.com/success",
        "webhook_details": {
            "webhook_version": "1.0.1",
            "webhook_username": "ekart_retail",
            "webhook_password": "password_ekart@123",
            "payment_created_enabled": True,
            "payment_succeeded_enabled": True,
            "payment_failed_enabled": True,
            "webhook_url": " https://cb30-13-232-74-226.ngrok.io"
        },
        "sub_merchants_enabled": False,
        "metadata": {
            "city": "NY",
            "unit": "245"
        }
    })
    headers = {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
        'feature': 'custom',
        'api-key': 'test_admin'
    }

    response = requests.request(
        "POST", acccount_url, headers=headers, data=payload)

    merchant_id = response.json()["merchant_id"]

    kv_enable_url = f"{url}/accounts/{merchant_id}/kv"

    payload = json.dumps({
        "kv_enabled": True
    })
    headers = {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
        'api-key': 'test_admin'
    }

    response = requests.request(
        "POST", kv_enable_url, headers=headers, data=payload)

    return merchant_id


def create_api_key(merchant_id):
    api_key_url = f"{url}/api_keys/{merchant_id}"
    payload = json.dumps({
        "name": "API Key 1",
        "description": None,
        "expiration": "2023-11-23T01:02:03.000Z"
    })

    headers = {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
        'api-key': 'test_admin'
    }

    response = requests.request(
        "POST", api_key_url, headers=headers, data=payload)

    return response.json()["api_key"]


def create_merchant_connector_account(merchant_id):
    mca_url = f"{url}/account/{merchant_id}/connectors"
    payload = json.dumps({
        "connector_type": "fiz_operations",
        "connector_name": "stripe_test",
        "connector_account_details": {
            "auth_type": "HeaderKey",
            "api_key": "xyz"
        },
        "test_mode": False,
        "disabled": False,
        "payment_methods_enabled": [
            {
                "payment_method": "card",
                "payment_method_types": [
                    {
                        "payment_method_type": "credit",
                        "minimum_amount": 1,
                        "maximum_amount": 68607706,
                        "recurring_enabled": True,
                        "installment_payment_enabled": True,
                        "card_networks": [
                            "Visa",
                            "Mastercard"
                        ]
                    }
                ]
            },
            {
                "payment_method": "card",
                "payment_method_types": [
                    {
                        "payment_method_type": "debit",
                        "minimum_amount": 1,
                        "maximum_amount": 68607706,
                        "recurring_enabled": True,
                        "installment_payment_enabled": True,
                        "card_networks": [
                            "Visa",
                            "Mastercard"
                        ]
                    }
                ]
            },
        ],

    })

    headers = {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
        'api-key': 'test_admin'
    }

    response = requests.request("POST", mca_url, headers=headers, data=payload)


def payments_create(api_key):
    payment_url = f"{url}/payments"

    payload = json.dumps({
        "amount": 600,
        "currency": "USD",
        "confirm": False,
        "capture_method": "automatic",
        "customer_id": "HScustomer1234",
        "email": "p143@example.com",
        "amount_to_capture": 600,
        "description": "Its my first payment request",
        "capture_on": "2022-09-10T10:11:12Z",
        "return_url": "https://google.com",
        "name": "Preetam",
        "phone": "999999999",
        "setup_future_usage": "off_session",
        "phone_country_code": "+65",
        "authentication_type": "no_three_ds",
        "payment_method": "card",
        "payment_method_type": "debit",
        "payment_method_data": {
            "card": {
                "card_number": "4242424242424242",
                "card_exp_month": "01",
                "card_exp_year": "2026",
                "card_holder_name": "joseph Doe",
                "card_cvc": "737"
            }
        },
        "connector_metadata": {
            "noon": {
                "order_category": "applepay"
            }
        },
        "browser_info": {
            "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/70.0.3538.110 Safari/537.36",
            "accept_header": "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8",
            "language": "nl-NL",
            "color_depth": 24,
            "screen_height": 723,
            "screen_width": 1536,
            "time_zone": 0,
            "java_enabled": True,
            "java_script_enabled": True,
            "ip_address": "128.0.0.1"
        },
        "billing": {
            "address": {
                "line1": "1467",
                "line2": "Harrison Street",
                "line3": "Harrison Street",
                "city": "San Fransico",
                "state": "California",
                "zip": "94122",
                "country": "US",
                "first_name": "preetam",
                "last_name": "revankar"
            },
            "phone": {
                "number": "8056594427",
                "country_code": "+91"
            }
        },
        "shipping": {
            "address": {
                "line1": "1467",
                "line2": "Harrison Street",
                "line3": "Harrison Street",
                "city": "San Fransico",
                "state": "California",
                "zip": "94122",
                "country": "US",
                "first_name": "joseph",
                "last_name": "Doe"
            },
            "phone": {
                "number": "8056594427",
                "country_code": "+91"
            }
        },
        "order_details": [
            {
                "product_name": "Apple iphone 15",
                "quantity": 1,
                "amount": 6540,
                "account_name": "transaction_processing"
            }
        ],
        "statement_descriptor_name": "joseph",
        "statement_descriptor_suffix": "JS",
        "metadata": {
            "udf1": "value1",
            "new_customer": "true",
            "login_date": "2019-09-10T10:11:12Z"
        }
    })
    headers = {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
        'api-key': api_key
    }

    response = requests.request(
        "POST", payment_url, headers=headers, data=payload)

    print(response.json())
    payment_ids.append(response.json()["payment_id"])


def payments_confirm(payment_id, api_key):
    confirm_url = f"{url}/payments/{payment_id}/confirm"

    payload = json.dumps({})
    headers = {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
        'api-key': api_key
    }

    response = requests.request(
        "POST", confirm_url, headers=headers, data=payload)
    print(response.json())


def main():
    try:
        to_confirm = open('payment_ids.txt', 'r').read().split(",")
    except Exception:
        to_confirm = None

    if to_confirm:
        api_key = open('api_key.txt', 'r').read()
        for payment_id in to_confirm:
            payments_confirm(payment_id, api_key)

        os.remove('api_key.txt')
        os.remove('payment_ids.txt')
    else:
        merchant_id = create_merchant_account()
        api_key = create_api_key(merchant_id)
        create_merchant_connector_account(merchant_id)
        with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
            arguments = [api_key] * 1000
            executor.map(payments_create, arguments)

        file_text = ",".join(payment_ids)
        payment_id_dump = open('payment_ids.txt', 'w')
        api_key_dump = open('api_key.txt', 'w')

        api_key_dump.write(api_key)
        payment_id_dump.write(file_text)


main()
