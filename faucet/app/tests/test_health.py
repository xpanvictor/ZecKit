"""
ZecKit Faucet - Health Endpoint Tests
Unit tests for health check functionality
"""
import pytest
from unittest.mock import Mock, patch
from app.main import create_app


@pytest.fixture
def app():
    """Create test app"""
    app = create_app('testing')
    return app


@pytest.fixture
def client(app):
    """Create test client"""
    return app.test_client()


@pytest.fixture
def mock_zebra_client():
    """Mock Zebra RPC client"""
    client = Mock()
    client.ping.return_value = True
    client.get_block_count.return_value = 200
    return client


@pytest.fixture
def mock_wallet():
    """Mock faucet wallet"""
    wallet = Mock()
    wallet.is_loaded.return_value = True
    wallet.get_balance.return_value = 1000.0
    wallet.get_address.return_value = "t1abc123..."
    return wallet


class TestHealthEndpoint:
    """Test suite for /health endpoint"""
    
    def test_health_check_all_healthy(self, app, client, mock_zebra_client, mock_wallet):
        """Test health check when all services are healthy"""
        app.zebra_client = mock_zebra_client
        app.faucet_wallet = mock_wallet
        
        response = client.get('/health')
        
        assert response.status_code == 200
        data = response.get_json()
        
        assert data['status'] == 'healthy'
        assert data['zebra_connected'] is True
        assert data['zebra_height'] == 200
        assert data['wallet_loaded'] is True
        assert data['balance'] == 1000.0
        assert data['version'] == '0.1.0'
        assert 'timestamp' in data
    
    def test_health_check_zebra_disconnected(self, app, client, mock_zebra_client, mock_wallet):
        """Test health check when Zebra is disconnected"""
        mock_zebra_client.ping.return_value = False
        app.zebra_client = mock_zebra_client
        app.faucet_wallet = mock_wallet
        
        response = client.get('/health')
        
        assert response.status_code == 503
        data = response.get_json()
        
        assert data['status'] == 'unhealthy'
        assert data['zebra_connected'] is False
        assert 'issues' in data
        assert 'zebra_not_responding' in data['issues']
    
    def test_health_check_wallet_not_loaded(self, app, client, mock_zebra_client, mock_wallet):
        """Test health check when wallet is not loaded"""
        mock_wallet.is_loaded.return_value = False
        app.zebra_client = mock_zebra_client
        app.faucet_wallet = mock_wallet
        
        response = client.get('/health')
        
        assert response.status_code == 503
        data = response.get_json()
        
        assert data['status'] == 'unhealthy'
        assert data['wallet_loaded'] is False
        assert 'issues' in data
        assert 'wallet_not_loaded' in data['issues']
    
    def test_health_check_low_balance(self, app, client, mock_zebra_client, mock_wallet):
        """Test health check with low balance warning"""
        mock_wallet.get_balance.return_value = 50.0  # Below default threshold of 100
        app.zebra_client = mock_zebra_client
        app.faucet_wallet = mock_wallet
        
        response = client.get('/health')
        
        assert response.status_code == 200
        data = response.get_json()
        
        assert data['status'] == 'degraded'
        assert data['balance'] == 50.0
        assert 'issues' in data
        assert any('low_balance' in issue for issue in data['issues'])
    
    def test_health_check_no_client(self, app, client):
        """Test health check when Zebra client not initialized"""
        # Don't set app.zebra_client
        app.faucet_wallet = Mock()
        app.faucet_wallet.is_loaded.return_value = True
        
        response = client.get('/health')
        
        assert response.status_code == 503
        data = response.get_json()
        
        assert data['status'] == 'unhealthy'
        assert 'issues' in data


class TestReadinessEndpoint:
    """Test suite for /ready endpoint"""
    
    def test_ready_when_services_ready(self, app, client, mock_zebra_client, mock_wallet):
        """Test readiness when services are ready"""
        app.zebra_client = mock_zebra_client
        app.faucet_wallet = mock_wallet
        
        response = client.get('/ready')
        
        assert response.status_code == 200
        data = response.get_json()
        assert data['ready'] is True
    
    def test_not_ready_when_zebra_down(self, app, client, mock_zebra_client, mock_wallet):
        """Test readiness when Zebra is down"""
        mock_zebra_client.ping.return_value = False
        app.zebra_client = mock_zebra_client
        app.faucet_wallet = mock_wallet
        
        response = client.get('/ready')
        
        assert response.status_code == 503
        data = response.get_json()
        assert data['ready'] is False
    
    def test_not_ready_when_wallet_not_loaded(self, app, client, mock_zebra_client, mock_wallet):
        """Test readiness when wallet not loaded"""
        mock_wallet.is_loaded.return_value = False
        app.zebra_client = mock_zebra_client
        app.faucet_wallet = mock_wallet
        
        response = client.get('/ready')
        
        assert response.status_code == 503
        data = response.get_json()
        assert data['ready'] is False


class TestLivenessEndpoint:
    """Test suite for /live endpoint"""
    
    def test_liveness_always_returns_200(self, client):
        """Test liveness always returns 200 if Flask is running"""
        response = client.get('/live')
        
        assert response.status_code == 200
        data = response.get_json()
        assert data['alive'] is True


class TestRootEndpoint:
    """Test suite for / endpoint"""
    
    def test_root_returns_service_info(self, client):
        """Test root endpoint returns service information"""
        response = client.get('/')
        
        assert response.status_code == 200
        data = response.get_json()
        
        assert data['service'] == 'ZecKit Faucet'
        assert data['version'] == '0.1.0'
        assert data['status'] == 'running'
        assert 'endpoints' in data
        assert '/health' in str(data['endpoints'])