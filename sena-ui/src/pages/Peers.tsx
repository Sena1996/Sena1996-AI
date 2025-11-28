import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Globe,
  RefreshCw,
  Check,
  X,
  Unplug,
  Edit2,
  Save,
  Users,
  Clock,
  Wifi,
  WifiOff,
} from 'lucide-react';
import clsx from 'clsx';
import { useToast } from '../components/Toast';

interface HubIdentity {
  hub_id: string;
  name: string;
  hostname: string;
  port: number;
  short_id: string;
}

interface ConnectedPeer {
  hub_id: string;
  name: string;
  address: string;
  port: number;
  is_online: boolean;
  session_count: number;
  connected_since: string;
}

interface ConnectionRequest {
  request_id: string;
  from_hub_id: string;
  from_hub_name: string;
  from_address: string;
  message: string | null;
  created_at: number;
}

export default function Peers() {
  const [identity, setIdentity] = useState<HubIdentity | null>(null);
  const [connectedPeers, setConnectedPeers] = useState<ConnectedPeer[]>([]);
  const [pendingRequests, setPendingRequests] = useState<ConnectionRequest[]>([]);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [editingName, setEditingName] = useState(false);
  const [newHubName, setNewHubName] = useState('');
  const toast = useToast();

  const loadData = useCallback(async () => {
    try {
      const [identityData, peersData, requestsData] = await Promise.all([
        invoke<HubIdentity>('get_hub_identity'),
        invoke<ConnectedPeer[]>('get_connected_peers'),
        invoke<ConnectionRequest[]>('get_pending_requests'),
      ]);
      setIdentity(identityData);
      setConnectedPeers(peersData);
      setPendingRequests(requestsData);
      setNewHubName(identityData.name);
    } catch (error) {
      console.error('Failed to load peer data:', error);
      toast.error(`Failed to load peer data: ${error}`);
    }
  }, [toast]);

  const refreshData = useCallback(async () => {
    setIsRefreshing(true);
    await loadData();
    setIsRefreshing(false);
    toast.success('Peer data refreshed');
  }, [loadData, toast]);

  const saveHubName = useCallback(async () => {
    if (!newHubName.trim()) {
      toast.error('Hub name cannot be empty');
      return;
    }
    try {
      await invoke('set_hub_name', { name: newHubName.trim() });
      setEditingName(false);
      await loadData();
      toast.success('Hub name updated');
    } catch (error) {
      toast.error(`Failed to update hub name: ${error}`);
    }
  }, [newHubName, loadData, toast]);

  const approveRequest = useCallback(async (requestId: string) => {
    try {
      await invoke('approve_peer_request', { requestId });
      await loadData();
      toast.success('Connection request approved');
    } catch (error) {
      toast.error(`Failed to approve request: ${error}`);
    }
  }, [loadData, toast]);

  const rejectRequest = useCallback(async (requestId: string) => {
    try {
      await invoke('reject_peer_request', { requestId });
      await loadData();
      toast.info('Connection request rejected');
    } catch (error) {
      toast.error(`Failed to reject request: ${error}`);
    }
  }, [loadData, toast]);

  const disconnectPeer = useCallback(async (hubId: string, hubName: string) => {
    try {
      await invoke('disconnect_peer', { hubId });
      await loadData();
      toast.success(`Disconnected from ${hubName}`);
    } catch (error) {
      toast.error(`Failed to disconnect: ${error}`);
    }
  }, [loadData, toast]);

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 10000);
    return () => clearInterval(interval);
  }, [loadData]);

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-dark-100">
            Cross-Hub Network
          </h1>
          <p className="text-dark-400 mt-1">
            Connect and collaborate with other SENA hubs
          </p>
        </div>
        <button
          onClick={refreshData}
          disabled={isRefreshing}
          className="btn-secondary"
          title="Refresh"
        >
          <RefreshCw className={clsx('w-5 h-5', isRefreshing && 'animate-spin')} />
        </button>
      </div>

      {identity && (
        <div className="card mb-6">
          <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
            <Globe className="w-5 h-5 text-sena-400" />
            This Hub Identity
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="text-xs text-dark-500 block mb-1">Hub Name</label>
              {editingName ? (
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    value={newHubName}
                    onChange={(e) => setNewHubName(e.target.value)}
                    className="input flex-1"
                    placeholder="Enter hub name"
                    onKeyDown={(e) => e.key === 'Enter' && saveHubName()}
                  />
                  <button onClick={saveHubName} className="btn-primary p-2">
                    <Save className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => {
                      setEditingName(false);
                      setNewHubName(identity.name);
                    }}
                    className="btn-secondary p-2"
                  >
                    <X className="w-4 h-4" />
                  </button>
                </div>
              ) : (
                <div className="flex items-center gap-2">
                  <span className="text-dark-100 font-medium">{identity.name}</span>
                  <button
                    onClick={() => setEditingName(true)}
                    className="text-dark-500 hover:text-dark-300 transition-colors"
                  >
                    <Edit2 className="w-4 h-4" />
                  </button>
                </div>
              )}
            </div>
            <div>
              <label className="text-xs text-dark-500 block mb-1">Hostname</label>
              <span className="text-dark-300">{identity.hostname}</span>
            </div>
            <div>
              <label className="text-xs text-dark-500 block mb-1">Hub ID</label>
              <span className="text-dark-400 font-mono text-sm">{identity.short_id}</span>
            </div>
            <div>
              <label className="text-xs text-dark-500 block mb-1">Network Port</label>
              <span className="text-dark-300">{identity.port}</span>
            </div>
          </div>
        </div>
      )}

      {pendingRequests.length > 0 && (
        <div className="card mb-6 border-yellow-500/30">
          <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
            <Clock className="w-5 h-5 text-yellow-400" />
            Pending Connection Requests
            <span className="badge badge-warning">{pendingRequests.length}</span>
          </h2>
          <div className="space-y-4">
            {pendingRequests.map((request) => (
              <PendingRequestCard
                key={request.request_id}
                request={request}
                onApprove={() => approveRequest(request.request_id)}
                onReject={() => rejectRequest(request.request_id)}
              />
            ))}
          </div>
        </div>
      )}

      <div className="card">
        <h2 className="text-lg font-semibold text-dark-100 mb-4 flex items-center gap-2">
          <Users className="w-5 h-5 text-sena-400" />
          Connected Hubs
          {connectedPeers.length > 0 && (
            <span className="badge badge-success">{connectedPeers.length}</span>
          )}
        </h2>

        {connectedPeers.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <div className="w-16 h-16 rounded-2xl bg-dark-800 flex items-center justify-center mb-4">
              <Globe className="w-8 h-8 text-dark-500" />
            </div>
            <h3 className="text-lg font-semibold text-dark-300">No Connected Hubs</h3>
            <p className="text-dark-500 mt-1 max-w-md">
              Other SENA hubs on your network will appear here once connected.
            </p>
            <div className="mt-6 p-4 rounded-lg bg-dark-800 text-left">
              <p className="text-xs text-dark-400 mb-2">Connect via CLI:</p>
              <code className="text-sm text-sena-400">
                sena peer connect &lt;hub-address&gt;
              </code>
            </div>
          </div>
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            {connectedPeers.map((peer) => (
              <ConnectedPeerCard
                key={peer.hub_id}
                peer={peer}
                onDisconnect={() => disconnectPeer(peer.hub_id, peer.name)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function PendingRequestCard({
  request,
  onApprove,
  onReject,
}: {
  request: ConnectionRequest;
  onApprove: () => void;
  onReject: () => void;
}) {
  const requestTime = new Date(request.created_at * 1000).toLocaleString();

  return (
    <div className="bg-dark-800 rounded-lg p-4 border border-yellow-500/20">
      <div className="flex items-start justify-between">
        <div>
          <h3 className="font-semibold text-dark-100">{request.from_hub_name}</h3>
          <p className="text-xs text-dark-500 mt-1">
            From: {request.from_address}
          </p>
          {request.message && (
            <p className="text-sm text-dark-300 mt-2 italic">
              "{request.message}"
            </p>
          )}
          <p className="text-xs text-dark-500 mt-2">
            Requested: {requestTime}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={onApprove}
            className="btn-primary p-2"
            title="Approve"
          >
            <Check className="w-4 h-4" />
          </button>
          <button
            onClick={onReject}
            className="btn-secondary p-2 hover:bg-red-500/20 hover:text-red-400"
            title="Reject"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  );
}

function ConnectedPeerCard({
  peer,
  onDisconnect,
}: {
  peer: ConnectedPeer;
  onDisconnect: () => void;
}) {
  return (
    <div className="bg-dark-800 rounded-lg p-4">
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-2">
          {peer.is_online ? (
            <Wifi className="w-4 h-4 text-green-400" />
          ) : (
            <WifiOff className="w-4 h-4 text-dark-500" />
          )}
          <h3 className="font-semibold text-dark-100">{peer.name}</h3>
          <span className={clsx(
            'badge',
            peer.is_online ? 'badge-success' : 'bg-dark-700 text-dark-400'
          )}>
            {peer.is_online ? 'Online' : 'Offline'}
          </span>
        </div>
        <button
          onClick={onDisconnect}
          className="text-dark-500 hover:text-red-400 transition-colors"
          title="Disconnect"
        >
          <Unplug className="w-4 h-4" />
        </button>
      </div>

      <div className="space-y-2 text-sm">
        <div className="flex items-center justify-between">
          <span className="text-dark-500">Address</span>
          <span className="text-dark-300">{peer.address}:{peer.port}</span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-dark-500">Sessions</span>
          <span className="text-dark-300">{peer.session_count}</span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-dark-500">Connected</span>
          <span className="text-dark-400">{peer.connected_since}</span>
        </div>
      </div>
    </div>
  );
}
