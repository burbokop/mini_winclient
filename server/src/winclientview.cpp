#include "winclientview.h"

#include <QPainter>
#include "winclient.h"

WinClientView::WinClientView(QQuickItem *parent)
    : QQuickPaintedItem{parent}
{
    sync();
}

WinClient *WinClientView::client() const
{
    return m_client;
}

void WinClientView::setClient(WinClient *newClient)
{
    if (m_client == newClient)
        return;
    m_client = newClient;
    sync();
    emit clientChanged();
}

void WinClientView::sync()
{
    for (const auto &c : m_conns) {
        disconnect(c);
    }
    m_conns.clear();

    if (m_client) {
        m_conns.push_back(connect(m_client, &WinClient::imageChanged, this, [this]() { update(); }));
        m_conns.push_back(connect(m_client, &WinClient::widthChanged, this, [this]() {
            setImplicitWidth(m_client->width());
        }));
        m_conns.push_back(connect(m_client, &WinClient::heightChanged, this, [this]() {
            setImplicitHeight(m_client->height());
        }));
    }
}

void WinClientView::paint(QPainter *painter)
{
    assert(painter);
    assert(m_client);
    painter->drawImage(0, 0, m_client->image());
}
