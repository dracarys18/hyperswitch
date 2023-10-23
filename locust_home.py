from email import header
from locust import HttpUser, TaskSet, SequentialTaskSet, task,  between
from uuid import uuid4 as u4
from random import choice
import json


class UserBehavior(SequentialTaskSet):
    payment_id = ""

    @task(1)
    def stripe(self):
        payload = json.dumps({
            "amount": 6540,
            "currency": "USD",
            "confirm": False,
            "business_country": "US",
            "business_label": "default",
            "capture_method": "automatic",
            "capture_on": "2022-09-10T10:11:12Z",
            "amount_to_capture": 6540,
            "customer_id": "StripeCustomer",
            "email": "guest@example.com",
            "name": "John Doe",
            "phone": "999999999",
            "phone_country_code": "+65",
            "description": "Its my first payment request",
            "authentication_type": "no_three_ds",
            "return_url": "https://duck.com",
            "billing": {
                "address": {
                    "line1": "1467",
                    "line2": "Harrison Street",
                    "line3": "Harrison Street",
                    "city": "San Fransico",
                    "state": "California",
                    "zip": "94122",
                    "country": "US",
                    "first_name": "PiX",
                    "last_name": "Pix",
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
                    "first_name": "PiX"
                }
            },
            "statement_descriptor_name": "joseph",
            "statement_descriptor_suffix": "JS",
            "metadata": {
                "udf1": "value1",
                "new_customer": "true",
                "login_date": "2019-09-10T10:11:12Z"
            }
        })
        response = self.client.post('/payments', data=payload,
                                    headers={
                                        'Content-Type': 'application/json',
                                        'Accept': 'application/json',
                                        'x-feature': 'router-custom',
                                        # 'api-key': 'xyz'
                                    })

        self.payment_id = json.loads(response.text)["payment_id"]
        print(self.payment_id)

    @task(1)
    def confimr(self):
        payload = json.dumps({"payment_method": "card",
                              "payment_method_type": "credit",
                              # "setup_future_usage": "on_session",
                              # "connector":["stripe_test"],
                              # "payment_method_data": {
                              #     "card": {
                              #       "card_number": "4200000000000000",
                              #       "card_exp_month": "10",
                              #       "card_exp_year": "25",
                              #       "card_holder_name": "joseph Doe",
                              #       "card_cvc": "123"
                              #     }
                              #   },
                              "payment_method_data": {
                                  "card": {
                                      "card_number": "4242424242424242",
                                      "card_exp_month": "10",
                                      "card_exp_year": "25",
                                      "card_holder_name": "joseph Doe",
                                      "card_cvc": "123"
                                  }
                              },
                              "browser_info": {
                                  "user_agent": "Mozilla\/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit\/537.36 (KHTML, like Gecko) Chrome\/70.0.3538.110 Safari\/537.36",
                                  "accept_header": "text\/html,application\/xhtml+xml,application\/xml;q=0.9,image\/webp,image\/apng,*\/*;q=0.8",
                                  "language": "nl-NL",
                                  "color_depth": 24,
                                  "screen_height": 723,
                                  "screen_width": 1536,
                                  "time_zone": 0,
                                  "java_enabled": True,
                                  "java_script_enabled": True,
                                  "ip_address": "125.0.0.1"
                              }})

        response = self.client.post('/payments/'+self.payment_id+'/confirm', name='/payments/payment_id/confirm', data=payload,
                                    headers={
                                        'Content-Type': 'application/json',
                                        'Accept': 'application/json',
                                        'x-feature': 'router-custom',
                                        # 'api-key': 'xyz'
                                    })

        print(json.loads(response.text))


class WebsiteUser(HttpUser):
    tasks = [UserBehavior]
    wait_time = between(0, 0)
    host = 'http://localhost:8080'
