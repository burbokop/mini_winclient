#include "winserver.h"

#include <QQmlInfo>
#include <QTcpServer>
#include "winclient.h"

MiniWinServer::MiniWinServer(QObject *parent)
    : QAbstractListModel{parent}
{
    connect(this, &MiniWinServer::portChanged, this, [this]() { listen(m_port); });
}

int MiniWinServer::rowCount(const QModelIndex &parent) const
{
    return m_clients.size();
}

QVariant MiniWinServer::data(const QModelIndex &index, int role) const
{
    if (index.row() >= 0 && index.row() < m_clients.size()) {
        return QVariant::fromValue(m_clients[index.row()]);
    } else {
        return QVariant();
    }
}

bool MiniWinServer::listen(quint16 port)
{
    if (m_server) {
        if (m_server->serverPort() == port)
            return false;

        if (m_verbose) {
            qInfo().nospace() << "Changing port: " << m_server->serverPort() << " -> " << port
                              << ".";
        }

        m_server->deleteLater();
    }

    beginResetModel();
    for (const auto &c : m_clients)
        c->deleteLater();
    m_clients.clear();
    endResetModel();

    m_server = new QTcpServer(this);
    connect(m_server, &QTcpServer::newConnection, this, [this]() {
        const auto client = new WinClient(m_server->nextPendingConnection(), m_nextId++, this);

        if (m_verbose) {
            qInfo().nospace() << "Client connected with id: " << client->id() << ".";
        }

        connect(client, &WinClient::dead, this, [this, client]() {
            if (m_verbose) {
                qInfo().nospace() << "Client with id " << client->id() << " is dead.";
            }

            const auto i = m_clients.indexOf(client);
            assert(i >= 0);
            beginRemoveRows(QModelIndex(), i, i);
            m_clients.removeAt(i);
            client->deleteLater();
            endRemoveRows();
        });

        beginInsertRows(QModelIndex(), m_clients.size(), m_clients.size());
        m_clients.push_back(client);
        endInsertRows();
    });

    if (!m_server->listen(QHostAddress::Any, port)) {
        qmlWarning(this) << "Listen failed: " << m_server->errorString();
        m_server->deleteLater();
        m_server = nullptr;
        return false;
    }

    if (m_verbose) {
        qInfo().nospace() << "Listening on port: " << m_server->serverPort()
                          << " (addr: " << m_server->serverAddress() << ")"
                          << ".";
    }

    return true;
}

void MiniWinServer::classBegin() {}

void MiniWinServer::componentComplete()
{
    if (!m_server) {
        listen(m_port);
    }
}
