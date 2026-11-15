import json
import tempfile
import unittest
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'backend'))

from google_calendar import (
    get_config,
    save_config,
    get_tokens,
    save_tokens,
    get_status,
    disconnect
)

class GoogleCalendarTests(unittest.TestCase):
    def test_config_and_status(self):
        with tempfile.TemporaryDirectory() as td:
            # Default status when empty
            status = get_status(td)
            self.assertFalse(status['connected'])
            self.assertEqual(status['email'], '')
            self.assertTrue(status['autoSync'])
            self.assertFalse(status['dismissPrompt'])

            # Save config
            save_config(td, {
                'clientId': 'test_client_id.apps.googleusercontent.com',
                'clientSecret': 'test_client_secret',
                'autoSync': False,
                'dismissPrompt': True
            })
            cfg = get_config(td)
            self.assertEqual(cfg['clientId'], 'test_client_id.apps.googleusercontent.com')
            self.assertFalse(cfg['autoSync'])
            self.assertTrue(cfg['dismissPrompt'])

            # Tokens simulation
            save_tokens(td, {
                'access_token': 'ya29.sample',
                'refresh_token': '1//sample_refresh',
                'email': 'mahasiswa@gmail.com',
                'expires_at': 9999999999
            })
            status2 = get_status(td)
            self.assertTrue(status2['connected'])
            self.assertEqual(status2['email'], 'mahasiswa@gmail.com')
            self.assertFalse(status2['autoSync'])
            self.assertTrue(status2['dismissPrompt'])

            # Disconnect
            disconnect(td)
            status3 = get_status(td)
            self.assertFalse(status3['connected'])
            self.assertEqual(status3['email'], '')

if __name__ == '__main__':
    unittest.main()
